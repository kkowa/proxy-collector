// use std::{net::SocketAddr, sync::Arc};

// use anyhow::{Error, Result};
// use futures::{SinkExt, StreamExt};
// use lazy_static::lazy_static;
// use lib::Proxy;
// use portpicker::pick_unused_port;
// use reqwest::{self, Client, StatusCode};
// use rstest::*;
// use rustls::ClientConfig;
// use tokio::{net::TcpStream, task};
// use tokio_tungstenite::{tungstenite, Connector, MaybeTlsStream, WebSocketStream};
// use tungstenite::Message;

// // TODO: Make a struct for WebSocket and WebSocketSecure client, like WSClient
// // TODO: Modularize certificate management for future works on it

// lazy_static! {
//     static ref ECHO_SERVER_HOST: String =
//         std::env::var("ECHO_SERVER_HOST").unwrap_or_else(|_| "localhost".to_string());
// }

// #[fixture]
// fn proxy() -> String {
//     let addr = SocketAddr::from((
//         [127, 0, 0, 1],
//         pick_unused_port().expect("no port available"),
//     ));

//     // FIXME: 127.0.0.1 fails with ConnectionRefused error; why?
//     let url = format!(
//         "http://{host}:{port}",
//         host = "localhost",
//         port = addr.port()
//     );

//     // Run server
//     task::spawn(async move {
//         let proxy = Proxy::builder().build();
//         proxy.run(&addr).await.unwrap()
//     });

//     url
// }

// #[fixture]
// fn http_client(proxy: String) -> Client {
//     // TODO: reqwest::Client to hyper with proxy; use hyper-proxy
//     Client::builder()
//         .danger_accept_invalid_certs(true)
//         .proxy(reqwest::Proxy::all(proxy).unwrap())
//         .build()
//         .unwrap()
// }

// pub struct NoCertificateVerification {}

// impl rustls::client::ServerCertVerifier for NoCertificateVerification {
//     fn verify_server_cert(
//         &self,
//         _end_entity: &rustls::Certificate,
//         _intermediates: &[rustls::Certificate],
//         _server_name: &rustls::ServerName,
//         _scts: &mut dyn Iterator<Item = &[u8]>,
//         _ocsp_response: &[u8],
//         _now: std::time::SystemTime,
//     ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
//         Ok(rustls::client::ServerCertVerified::assertion())
//     }
// }

// async fn wss_client(
//     proxy: String,
//     server: http::Uri,
// ) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
//     let uri = proxy
//         .parse::<http::Uri>()
//         .unwrap_or_else(|_| panic!("{}", format!("{proxy} is not valid uri")));

//     let authority = uri.authority().unwrap().to_string();
//     let mut stream = tokio::net::TcpStream::connect(authority).await.unwrap();

//     async_http_proxy::http_connect_tokio(
//         &mut stream,
//         server.host().unwrap(),
//         server.port_u16().unwrap_or(80),
//     )
//     .await
//     .unwrap();

//     let mut client_config = ClientConfig::builder()
//         .with_safe_defaults()
//         .with_root_certificates(rustls::RootCertStore::empty())
//         .with_no_client_auth();

//     client_config
//         .dangerous()
//         .set_certificate_verifier(Arc::new(NoCertificateVerification {}));

//     let connector = Connector::Rustls(Arc::new(client_config));
//     let (ws, _) =
//         tokio_tungstenite::client_async_tls_with_config(server, stream, None, Some(connector))
//             .await
//             .unwrap();

//     ws
// }

// #[rstest]
// #[tokio::test]
// async fn http_proxy(http_client: Client) -> Result<(), Error> {
//     let res = http_client
//         .get(format!("http://{}:10080/", ECHO_SERVER_HOST.as_str()))
//         .send()
//         .await?;

//     assert_eq!(res.status(), StatusCode::OK);
//     assert!(res.text().await?.starts_with("Welcome to echo-server!"));

//     Ok(())
// }

// #[rstest]
// #[tokio::test]
// async fn https_tunnel(http_client: Client) -> Result<(), Error> {
//     let res = http_client
//         .get(format!("https://{}:10443/", ECHO_SERVER_HOST.as_str()))
//         .send()
//         .await?;

//     assert_eq!(res.status(), StatusCode::OK);
//     assert!(res.text().await?.contains("TLS Connection Info"));

//     Ok(())
// }

// #[rstest]
// #[tokio::test]
// async fn ws_tunnel(proxy: String) -> Result<(), Error> {
//     let server = format!("ws://{}:10080", ECHO_SERVER_HOST.as_str()).parse::<http::Uri>()?;
//     let mut ws = wss_client(proxy, server).await;
//     ws.send(Message::Text("aHB<j3{iSa%5MR@!L!rA".to_string()))
//         .await?;

//     // Ignore first line "Request served by ..."
//     let _ignore = ws.next().await;
//     let msg = ws.next().await.unwrap()?.to_string();
//     ws.close(None).await?;

//     assert_eq!(msg, "aHB<j3{iSa%5MR@!L!rA");

//     Ok(())
// }

// #[rstest]
// #[tokio::test]
// async fn wss_tunnel(proxy: String) -> Result<(), Error> {
//     let server = format!("wss://{}:10443", ECHO_SERVER_HOST.as_str()).parse::<http::Uri>()?;
//     let mut ws = wss_client(proxy, server).await;
//     ws.send(Message::Text("(XU(6R>A6jJ?8yLV(AJn".to_string()))
//         .await?;

//     // Ignore first line "Request served by ..."
//     let _ignore = ws.next().await;
//     let msg = ws.next().await.unwrap()?.to_string();
//     ws.close(None).await?;

//     assert_eq!(msg, "(XU(6R>A6jJ?8yLV(AJn");

//     Ok(())
// }
