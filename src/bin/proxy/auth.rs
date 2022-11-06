//! Main binary for use by kkowa application system.

use anyhow::Result;
use async_trait::async_trait;
use kkowa_proxy::{auth::{Authenticator, Credentials},
                  http::Uri};
use server_openapi::apis::{configuration::Configuration, users_api::users_me_api_users_me_get};
use tracing::{debug, trace};

#[derive(Debug)]
pub struct ServerAuth {
    uri: Option<Uri>,
}

impl ServerAuth {
    pub fn new(uri: Option<Uri>) -> Self {
        Self { uri }
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

            let cfg = Configuration {
                base_path: u.to_string().trim_end_matches('/').to_string(),
                bearer_access_token: Some(credentials.credentials().to_string()),
                ..Configuration::default()
            };

            let result = users_me_api_users_me_get(&cfg).await;
            match result {
                Ok(user) => {
                    debug!("authentication succeed, user {}", user.username);
                    return Ok(());
                }
                Err(err) => {
                    debug!("auth failure: {err:#?}")
                }
            }
        }

        Err(kkowa_proxy::auth::Error::NotAuthenticated)
    }
}

#[cfg(test)]
mod tests {}
