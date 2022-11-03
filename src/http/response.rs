use anyhow::Result;

use super::{HeaderName, HeaderValue, Headers, Payload, Request, StatusCode, Version};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Response {
    pub status: StatusCode,
    pub version: Version,
    pub headers: Headers,
    pub payload: Payload,

    /// Source request to current response.
    pub request: Request,
}

impl Response {
    /// Create new response with default params and request.
    pub fn new<P>(
        status: StatusCode,
        version: Version,
        headers: Headers,
        payload: P,
        request: Request,
    ) -> Self
    where
        P: Into<Payload>,
    {
        Self {
            status,
            version,
            headers,
            payload: payload.into(),
            request,
        }
    }

    pub fn builder() -> Builder {
        Builder::new()
    }

    pub async fn from<R>(resp: hyper::Response<hyper::body::Body>, request: R) -> Result<Self>
    where
        R: Into<Request>,
    {
        let (parts, body) = resp.into_parts();
        let bytes = hyper::body::to_bytes(body).await?;

        Ok(Self::new(
            parts.status,
            parts.version,
            parts.headers,
            bytes,
            request.into(),
        ))
    }
}

impl From<Response> for hyper::Response<hyper::Body> {
    fn from(val: Response) -> Self {
        let mut builder = hyper::Response::builder()
            .status(val.status)
            .version(val.version);

        *(builder.headers_mut().unwrap()) = val.headers;

        builder.body(val.payload.into()).unwrap()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Builder {
    pub status: StatusCode,
    pub version: Version,
    pub headers: Headers,
    pub payload: Payload,

    /// Source request to current response.
    pub request: Request,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    pub fn version(mut self, version: Version) -> Self {
        self.version = version;
        self
    }

    pub fn header(mut self, key: HeaderName, value: HeaderValue) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn headers(mut self, headers: Headers) -> Self {
        self.headers.extend(headers);
        self
    }

    pub fn payload(mut self, payload: Payload) -> Self {
        self.payload = payload;
        self
    }

    pub fn request(mut self, request: Request) -> Self {
        self.request = request;
        self
    }

    pub fn build(self) -> Response {
        Response::new(
            self.status,
            self.version,
            self.headers,
            self.payload,
            self.request,
        )
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use http::{StatusCode, Version};

    use super::{Request, Response};

    #[tokio::test]
    async fn response_from_hyper() -> Result<()> {
        let hyper_resp = hyper::Response::new(hyper::Body::from("Good Evening"));
        let resp = Response::from(hyper_resp, Request::default()).await?;

        assert_eq!(resp.status, StatusCode::OK);
        assert_eq!(resp.version, Version::HTTP_11);
        assert!(resp.headers.is_empty());
        assert_eq!(resp.payload, b"Good Evening");

        Ok(())
    }

    #[tokio::test]
    async fn response_into_hyper() -> Result<()> {
        let resp = Response::default();
        let hyper_resp: hyper::Response<hyper::Body> = resp.into();

        assert_eq!(hyper_resp.status(), StatusCode::OK);
        assert_eq!(hyper_resp.version(), Version::HTTP_11);
        assert!(hyper_resp.headers().is_empty());
        assert!(hyper_resp.extensions().is_empty());
        assert_eq!(hyper::body::to_bytes(hyper_resp.into_body()).await?, vec![]);

        Ok(())
    }
}
