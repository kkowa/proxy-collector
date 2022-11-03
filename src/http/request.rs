use anyhow::Result;

use super::{HeaderName, HeaderValue, Headers, Method, Payload, Uri, Version};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub headers: Headers,
    pub payload: Payload,
}

impl Request {
    /// Create new request with default params.
    pub fn new<P>(method: Method, uri: Uri, version: Version, headers: Headers, payload: P) -> Self
    where
        P: Into<Payload>,
    {
        Self {
            method,
            uri,
            version,
            headers,
            payload: payload.into(),
        }
    }

    pub fn builder() -> Builder {
        Builder::new()
    }

    pub async fn from(req: hyper::Request<hyper::Body>) -> Result<Self> {
        let (parts, body) = req.into_parts();
        let bytes = hyper::body::to_bytes(body).await?;

        Ok(Self::new(
            parts.method,
            parts.uri,
            parts.version,
            parts.headers,
            bytes,
        ))
    }
}

impl From<Request> for hyper::Request<hyper::Body> {
    fn from(val: Request) -> Self {
        let mut builder = hyper::Request::builder()
            .method(val.method)
            .uri(val.uri)
            .version(val.version);

        *(builder.headers_mut().unwrap()) = val.headers;

        builder.body(val.payload.into()).unwrap()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Builder {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub headers: Headers,
    pub payload: Payload,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    pub fn uri(mut self, uri: Uri) -> Self {
        self.uri = uri;
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

    pub fn build(self) -> Request {
        Request::new(
            self.method,
            self.uri,
            self.version,
            self.headers,
            self.payload,
        )
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use http::{Method, Uri, Version};

    use super::Request;

    #[tokio::test]
    async fn request_from_hyper() -> Result<()> {
        let hyper_req = hyper::Request::new(hyper::Body::from("Hello World!"));
        let req = Request::from(hyper_req).await?;

        assert_eq!(req.method, Method::GET);
        assert_eq!(req.uri, Uri::from_static("/"));
        assert_eq!(req.version, Version::HTTP_11);
        assert!(req.headers.is_empty());
        assert_eq!(req.payload, b"Hello World!");

        Ok(())
    }

    #[tokio::test]
    async fn request_into_hyper() -> Result<()> {
        let req = Request::default();
        let hyper_req: hyper::Request<hyper::body::Body> = req.into();

        assert_eq!(*hyper_req.method(), Method::GET);
        assert_eq!(*hyper_req.uri(), Uri::from_static("/"));
        assert_eq!(hyper_req.version(), Version::HTTP_11);
        assert!(hyper_req.headers().is_empty());
        assert!(hyper_req.extensions().is_empty());
        assert_eq!(hyper::body::to_bytes(hyper_req.into_body()).await?, vec![]);

        Ok(())
    }
}
