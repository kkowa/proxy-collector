//! Report handler module sending processed JSON documents to API endpoint

mod processor;

use async_trait::async_trait;
use kkowa_proxy_lib::{http::{Response, Uri},
                      proxy::{Flow, Handler, Reverse}};
use serde_json::json;
use server_openapi::{apis::{configuration::Configuration,
                            documents_api::create_documents_api_documents_post},
                     models::CreateDocument};
use tracing::debug;

pub use self::processor::Processor;

/// Handler for collecting processed documents and uploading to remote server.
#[derive(Debug)]
pub struct Collector {
    /// API endpoint URL to POST docs.
    upload_to: Option<Uri>,

    /// Document processor.
    processors: Vec<Processor>,
}

impl Collector {
    /// Create new handler.
    pub fn new(upload_to: Option<Uri>, processors: Vec<Processor>) -> Self {
        Self {
            upload_to,
            processors,
        }
    }

    /// Generate document from HTTP flow.
    fn process(&self, resp: &Response) -> CreateDocument {
        let mut documents = Vec::with_capacity(self.processors.len());
        for processor in &self.processors {
            match processor.process(resp) {
                Some(document) => documents.push(document),
                None => {
                    debug!("document process returned nothing");
                }
            }
        }

        CreateDocument {
            folder: "temp".to_string(), // TODO: Each item should be categorized by something (URL or something)
            data: Some(Some(json!(documents))),
        }
    }
}

#[async_trait]
impl Handler for Collector {
    async fn on_response(&self, flow: &Flow, resp: Response) -> Reverse {
        if let Some(u) = &self.upload_to {
            let u = u.clone();
            if let Some(credentials) = flow.auth() {
                let credentials = credentials.clone();
                let create_document = self.process(&resp);

                tokio::task::spawn(async move {
                    let cfg = Configuration {
                        base_path: u.to_string().trim_end_matches('/').to_string(),
                        bearer_access_token: Some(credentials.credentials().to_string()),
                        ..Configuration::default()
                    };
                    create_documents_api_documents_post(&cfg, vec![create_document]).await
                });
            }
        }

        Reverse::DoNothing
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use httpmock::prelude::*;
    use kkowa_proxy_lib::http::{Headers, Method, Request, Response, StatusCode, Uri, Version};
    use rstest::*;
    use serde_json::json;
    use server_openapi::models::CreateDocument;

    use super::{Collector, Processor};

    struct Fixture {
        mock_server: MockServer,
        handler: Collector,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let mock_server = MockServer::start();
        let handler = Collector::new(
            Some(Uri::from_str(&mock_server.url("/report")).unwrap()),
            vec![Processor::from_str(include_str!("./donuts-processor.yaml")).unwrap()],
        );

        Fixture {
            mock_server,
            handler,
        }
    }

    #[rstest]
    fn handler_process(fixture: Fixture) {
        let req = Request::new(
            Method::GET,
            Uri::from_static("http://subdomain.domain.com/donuts"),
            Version::HTTP_11,
            Headers::new(),
            vec![],
        );

        let resp = Response::new(
            StatusCode::OK,
            Version::HTTP_11,
            Headers::new(),
            include_bytes!("./donuts.json").to_vec(),
            req,
        );
        let create_document = fixture.handler.process(&resp);

        assert_eq!(
            create_document,
            CreateDocument {
                folder: "temp".to_string(),
                data: Some(Some(json!([{
                    "extracted": {
                        "donutNames": ["Cake", "Raised", "Old Fashioned"]
                    }
                }])))
            }
        );
    }

    #[rstest]
    fn handler_process_no_match(fixture: Fixture) {
        let req = Request::new(
            Method::GET,
            Uri::from_static("http://subdomain.domain-idk.com/donuts"),
            Version::HTTP_11,
            Headers::new(),
            vec![],
        );

        let resp = Response::new(
            StatusCode::OK,
            Version::HTTP_11,
            Headers::new(),
            include_bytes!("./donuts.json").to_vec(),
            req,
        );

        let create_document = fixture.handler.process(&resp);

        assert_eq!(
            create_document,
            CreateDocument {
                folder: "temp".to_string(),
                data: Some(Some(json!([])))
            }
        );
    }
}
