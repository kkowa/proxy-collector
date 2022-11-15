//! Main binary for use by kkowa application system.

use anyhow::Result;
use async_trait::async_trait;
use kkowa_proxy_lib::{auth::{Authenticator, Credentials, Error},
                      http::Uri};
use server_openapi::apis::{configuration::Configuration, users_api::users_me_api_users_me_get};
use tracing::{debug, trace};

#[derive(Debug)]
pub struct Delegator {
    uri: Option<Uri>,
}

impl Delegator {
    pub fn new(uri: Option<Uri>) -> Self {
        Self { uri }
    }
}

#[async_trait]
impl Authenticator for Delegator {
    async fn authenticate(&self, credentials: &Credentials) -> Result<(), Error> {
        if let Some(u) = &self.uri {
            if credentials.scheme().to_lowercase() != "bearer" {
                trace!(
                    "scheme expected \"bearer\" but got \"{got}\"",
                    got = credentials.scheme()
                );
                return Err(Error::InvalidScheme {
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
        Err(Error::NotAuthenticated)
    }
}

#[cfg(test)]
mod tests {
    use httpmock::prelude::*;
    use kkowa_proxy_lib::auth::{Authenticator, Credentials, Error};
    use rstest::*;
    use server_openapi::models::User;

    use super::Delegator;

    #[fixture]
    fn server() -> MockServer {
        MockServer::start()
    }

    #[rstest]
    #[tokio::test]
    async fn delegator(server: MockServer) {
        // Mock server response for auth success
        server.mock(|when, then| {
            when.header_exists("Authorization");
            then.status(200).json_body(
                serde_json::to_value(User {
                    username: "user".to_string(),
                    email: "email@email.email".to_string(),
                    date_joined: "2022-05-19T05:29:21.847Z".to_string(),
                    last_login: None,
                })
                .unwrap(),
            );
        });
        assert!(Delegator::new(Some(server.url("").parse().unwrap()))
            .authenticate(&Credentials::new("Bearer", "TOKEN"))
            .await
            .is_ok());
    }

    #[rstest]
    #[tokio::test]
    async fn delegator_invalid_scheme(server: MockServer) {
        server.mock(|when, then| {
            when.any_request();
            then.status(500);
        });
        assert!(matches!(
            Delegator::new(Some(server.url("").parse().unwrap()))
                .authenticate(&Credentials::new("Basic", "dXNlcm5hbWU6cGFzc3dvcmQ=")) // username:password
                .await,
            Err(Error::InvalidScheme { .. })
        ));
    }

    #[rstest]
    #[tokio::test]
    async fn delegator_unauthenticated(server: MockServer) {
        server.mock(|when, then| {
            when.any_request();
            then.status(401);
        });
        assert!(matches!(
            Delegator::new(Some(server.url("").parse().unwrap()))
                .authenticate(&Credentials::new("Bearer", "TOKEN"))
                .await,
            Err(Error::NotAuthenticated)
        ))
    }
}
