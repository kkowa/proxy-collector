use std::fmt::Debug;

use thiserror::Error;
use tracing::{debug, trace};

use crate::http::{header, Request};

#[derive(Debug, Error)]
pub enum Error {
    #[error("required authorization header does not exists in request")]
    MissingHeader,

    #[error("failed to parse provided data into desired format")]
    InvalidFormat { n: usize },

    #[error("unknown error")]
    Unknown,
}

// TODO: Implement its own Debug / Fmt traits to mask credentials for security
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Credentials {
    scheme: String,
    credentials: String,
}

impl Credentials {
    pub fn new<S>(scheme: S, credentials: S) -> Self
    where
        S: AsRef<str>,
    {
        Self {
            scheme: scheme.as_ref().to_string(),
            credentials: credentials.as_ref().to_string(),
        }
    }

    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    pub fn credentials(&self) -> &str {
        &self.credentials
    }

    /// Create object from request's proxy authorization header.
    pub fn get_from_request(request: &Request) -> Result<Self, Error> {
        match request.headers.get(header::PROXY_AUTHORIZATION) {
            Some(value) => {
                let arr: [&str; 2] = value
                    .to_str()
                    .expect("failed to convert header value to string")
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .try_into()
                    .map_err(|v: Vec<&str>| Error::InvalidFormat { n: v.len() })?;

                let (scheme, credentials) = (arr[0], arr[1]);
                trace!(
                    "parsed credentials with scheme {scheme} and {n}-length credentials data",
                    n = credentials.len()
                );

                Ok(Credentials::new(scheme, credentials))
            }
            None => {
                {
                    let keys = request
                        .headers
                        .keys()
                        .map(|k| k.as_str())
                        .collect::<Vec<&str>>()
                        .join(", ");

                    debug!(
                        "required header \"Proxy-Authorization\" does not exists, there: {keys}",
                    );
                };

                Err(Error::MissingHeader)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::Credentials;
    use crate::http::{header, Request};

    #[test]
    fn get_from_request() -> Result<()> {
        let req = Request::builder()
            .header(
                header::PROXY_AUTHORIZATION,
                "Basic dXNlcm5hbWU6cGFzc3dvcmQ=".parse().unwrap(),
            )
            .build();

        assert_eq!(
            Credentials::get_from_request(&req)?,
            Credentials::new("Basic", "dXNlcm5hbWU6cGFzc3dvcmQ=")
        );

        Ok(())
    }

    #[test]
    fn get_from_request_header_not_set() -> Result<()> {
        let err = Credentials::get_from_request(&Request::default())
            .err()
            .unwrap();

        assert!(matches!(err, super::Error::MissingHeader));

        Ok(())
    }

    #[test]
    fn get_from_request_fields_lacking() -> Result<()> {
        let req = Request::builder()
            .header(header::PROXY_AUTHORIZATION, "Scheme".parse().unwrap())
            .build();
        let err = Credentials::get_from_request(&req).err().unwrap();

        assert!(matches!(err, super::Error::InvalidFormat { n: 1, .. }));

        Ok(())
    }

    #[test]
    fn get_from_request_too_many_fields() -> Result<()> {
        let req = Request::builder()
            .header(
                header::PROXY_AUTHORIZATION,
                "Scheme Value Extra".parse().unwrap(),
            )
            .build();
        let err = Credentials::get_from_request(&req).err().unwrap();

        assert!(matches!(err, super::Error::InvalidFormat { n: 3, .. }));

        Ok(())
    }
}
