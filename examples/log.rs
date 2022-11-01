use std::net::SocketAddr;

use async_trait::async_trait;
use kkowa_proxy::{http::{Request, Response},
                  proxy::{Flow, Forward, Handler, Reverse}};
use tracing::{debug, warn};

#[derive(Debug)]
pub struct LogHandler;

#[async_trait]
impl Handler for LogHandler {
    async fn on_request(&self, _ctx: &Flow, req: Request) -> Forward {
        debug!(
            "Seeing a request: {:?} {} {}",
            req.version, req.method, req.uri
        );

        Forward::DoNothing
    }

    async fn on_response(&self, _ctx: &Flow, resp: Response) -> Reverse {
        debug!("Seeing a response: {:?} {}", resp.version, resp.status);

        Reverse::DoNothing
    }
}

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("failed to set global default tracing subscriber");

    let addr: SocketAddr = "0.0.0.0:8080".parse().expect("failed to parse address");

    warn!("Listening on {}", addr);
    let app = kkowa_proxy::Proxy::new(
        "logging-proxy",
        hyper::Client::default(),
        vec![],
        vec![Box::new(LogHandler)],
    );

    app.run(&addr).await.expect("error occurred from server");
}
