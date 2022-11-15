use std::{convert::Infallible, net::SocketAddr};

use hyper::{header,
            service::{make_service_fn, service_fn},
            Error, Method, StatusCode};
use metrics_exporter_prometheus::PrometheusHandle;
use once_cell::sync::OnceCell;
use tracing::info;

pub(crate) static METRICS_HANDLE: OnceCell<PrometheusHandle> = OnceCell::new();

/// HTTP server instance for internal purpose, such as serving health checks, metrics, etc.
#[derive(Clone, Default)]
pub struct Web {}

impl Web {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&self, addr: &SocketAddr) -> Result<(), Error> {
        let make_service = make_service_fn(move |_| async move {
            let service = service_fn(serve);

            Ok::<_, Infallible>(service)
        });

        hyper::Server::bind(addr)
            .serve(make_service)
            .with_graceful_shutdown(self.graceful_shutdown())
            .await
    }

    async fn graceful_shutdown(&self) {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C signal handler");

        // Do shutdown tasks here
    }
}

async fn serve(
    req: hyper::Request<hyper::Body>,
) -> Result<hyper::Response<hyper::Body>, hyper::Error> {
    let (version, method, uri) = (
        req.version(),
        req.method().to_owned(),
        req.uri().to_string(),
    );
    let uri = uri.as_str();
    info!("{version:?} {method} {uri}");

    // Simple route implementation

    match (method, uri) {
        // GET /(ht|healthz)
        (Method::GET, "/ht" | "/healthz") => healthz().await,

        // GET /metrics
        (Method::GET, "/metrics") => metrics(METRICS_HANDLE.get()).await,

        // Fallback
        (_, _) => not_found().await,
    }
}

async fn metrics(
    handle: Option<&PrometheusHandle>,
) -> Result<hyper::Response<hyper::Body>, hyper::Error> {
    let response = match handle {
        Some(h) => hyper::Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/plain")
            .body(h.render().into())
            .unwrap(),
        None => hyper::Response::builder()
            .status(StatusCode::NOT_IMPLEMENTED)
            .body(hyper::body::Body::empty())
            .unwrap(),
    };

    Ok(response)
}

async fn healthz() -> Result<hyper::Response<hyper::Body>, hyper::Error> {
    Ok(hyper::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain")
        .body("OK".into())
        .unwrap())
}

async fn not_found() -> Result<hyper::Response<hyper::Body>, hyper::Error> {
    Ok(hyper::Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body("Not found".into())
        .unwrap())
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use hyper::{body::to_bytes, StatusCode};

    #[tokio::test]
    async fn healthz() -> Result<()> {
        let resp = super::healthz().await?;

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(to_bytes(resp.into_body()).await?.to_vec(), b"OK");

        Ok(())
    }

    #[tokio::test]
    async fn metrics() -> Result<()> {
        // NOTE: Creating handle manually, due to unknown test failures using init_metrics()
        let handle = metrics_exporter_prometheus::PrometheusBuilder::new()
            .build_recorder()
            .handle();

        let resp = super::metrics(Some(&handle)).await?;

        assert_eq!(resp.status(), StatusCode::OK);

        // NOTE: Body check skipped because it requires recorder installation, which causes error described above

        Ok(())
    }

    #[tokio::test]
    async fn not_found() -> Result<()> {
        let resp = super::not_found().await?;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        assert_eq!(to_bytes(resp.into_body()).await?.to_vec(), b"Not found");

        Ok(())
    }
}
