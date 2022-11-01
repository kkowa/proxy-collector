//! Main binary for use by kkowa application system.

use anyhow::Result;
use async_trait::async_trait;
use kkowa_proxy::{auth::{Authenticator, Credentials},
                  http::{header, Method, Uri}};
use tracing::{debug, trace};

type Client = hyper::Client<hyper::client::HttpConnector>;

#[derive(Debug)]
pub struct ServerAuth {
    uri: Option<Uri>,
    client: Client,
}

impl ServerAuth {
    pub fn new(uri: Option<Uri>) -> Self {
        Self {
            uri,
            client: Client::default(),
        }
    }
}

#[async_trait]
impl Authenticator for ServerAuth {
    async fn authenticate(
        &self,
        credentials: &Credentials,
    ) -> Result<(), kkowa_proxy::auth::Error> {
        if let Some(u) = &self.uri {
            if credentials.scheme().to_lowercase() != "bearer" {
                trace!(
                    "scheme expected \"bearer\" but got \"{got}\"",
                    got = credentials.scheme()
                );
                return Err(kkowa_proxy::auth::Error::InvalidScheme {
                    got: credentials.scheme().to_string(),
                    expect: "bearer".to_string(),
                });
            }

            let req = hyper::Request::builder()
                .method(Method::GET)
                .uri(u)
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {token}", token = credentials.credentials()),
                )
                .body(hyper::Body::empty())
                .unwrap();

            if let Ok(resp) = self.client.request(req).await {
                let status = resp.status();
                debug!("auth server responded with status code {status}",);

                if status.is_success() {
                    return Ok(());
                }
            };
        }

        Err(kkowa_proxy::auth::Error::NotAuthenticated)
    }
}

#[cfg(test)]
mod tests {}
