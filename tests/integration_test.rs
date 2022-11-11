use std::net::SocketAddr;

use anyhow::{Error, Result};
use lib::Proxy;
use portpicker::pick_unused_port;
use reqwest::{self, Client, StatusCode};
use rstest::*;

// TODO: Make a struct for WebSocket and WebSocketSecure client, like WSClient
// TODO: Modularize certificate management for future works on it

#[fixture]
fn proxy() -> String {
    let addr = SocketAddr::from((
        [127, 0, 0, 1],
        pick_unused_port().expect("no port available"),
    ));

    // FIXME: 127.0.0.1 fails with ConnectionRefused error; why?
    let url = format!(
        "http://{host}:{port}",
        host = "localhost",
        port = addr.port()
    );

    // Run server
    tokio::task::spawn(async move {
        let proxy = Proxy::builder().build().unwrap();
        proxy.run(&addr).await.unwrap()
    });

    url
}

#[fixture]
fn http_client(proxy: String) -> Client {
    Client::builder()
        .danger_accept_invalid_certs(true)
        .proxy(reqwest::Proxy::all(proxy).unwrap())
        .build()
        .unwrap()
}

pub struct NoCertificateVerification {}

impl rustls::client::ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

#[rstest]
#[tokio::test]
async fn http_proxy(http_client: Client) -> Result<(), Error> {
    let res = http_client.get("http://httpbin.org/get").send().await?;

    assert_eq!(res.status(), StatusCode::OK);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn https_tunnel(http_client: Client) -> Result<(), Error> {
    let res = http_client.get("https://httpbin.org/get").send().await?;

    assert_eq!(res.status(), StatusCode::OK);

    Ok(())
}
