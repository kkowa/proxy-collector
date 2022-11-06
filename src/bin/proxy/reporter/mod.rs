//! Report handler module sending processed JSON documents to API endpoint

mod processor;

use async_trait::async_trait;
use kkowa_proxy::{http::{Response, Uri},
                  proxy::{Flow, Handler, Reverse}};
use serde_json::json;
use server_openapi::{apis::{configuration::Configuration,
                            documents_api::create_documents_api_documents_post},
                     models::CreateDocument};
use tracing::debug;

pub use self::processor::Processor;

/// Handler for reporting processed documents to remote server.
#[derive(Debug)]
pub struct Reporter {
    /// API endpoint URL to POST docs.
    report_to: Option<Uri>,

    /// Document processor.
    processors: Vec<Processor>,
}

impl Reporter {
    /// Create new handler.
    pub fn new(report_to: Option<Uri>, processors: Vec<Processor>) -> Self {
        Self {
            report_to,
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
                    documents.push(json!({}))
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
impl Handler for Reporter {
    async fn on_response(&self, flow: &Flow, resp: Response) -> Reverse {
        if let Some(u) = &self.report_to {
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
    // use std::str::FromStr;

    // use anyhow::{Error, Result};
    // use httpmock::prelude::*;
    // use kkowa_proxy::http::{StatusCode, Uri};
    // use rstest::*;
    // use serde_json::json;

    // use super::{Processor, Reporter};

    // struct Fixture {
    //     mock_server: MockServer,
    //     handler: Reporter,
    // }

    // #[fixture]
    // fn fixture() -> Fixture {
    //     let mock_server = MockServer::start();
    //     let handler = Reporter::new(
    //         Some(Uri::from_str(&mock_server.url("/report")).unwrap()),
    //         vec![Processor::from_str(include_str!("./donuts-processor.yaml")).unwrap()],
    //     );

    //     Fixture {
    //         mock_server,
    //         handler,
    //     }
    // }

    // #[rstest]
    // fn handler_process(fixture: Fixture) {
    //     let req = Request::new(
    //         Method::GET,
    //         Uri::from_static("http://subdomain.domain.com/donuts"),
    //         Version::HTTP_11,
    //         Headers::new(),
    //         vec![],
    //     );

    //     let resp = Response::new(
    //         StatusCode::OK,
    //         Version::HTTP_11,
    //         Headers::new(),
    //         include_bytes!("./donuts.json").to_vec(),
    //         req,
    //     );
    //     let report = fixture.handler.process(&resp);

    //     assert_eq!(
    //         report,
    //         json!([
    //             {
    //                 "folder": "temp",
    //                 "data": [
    //                     {
    //                         "extracted": {
    //                             "donutNames": ["Cake", "Raised", "Old Fashioned"]
    //                         }
    //                     }
    //                 ]
    //             }
    //         ])
    //     );
    // }

    // #[rstest]
    // fn handler_process_no_match(fixture: Fixture) {
    //     let req = Request::new(
    //         Method::GET,
    //         Uri::from_static("http://subdomain.domain-idk.com/donuts"),
    //         Version::HTTP_11,
    //         Headers::new(),
    //         vec![],
    //     );

    //     let resp = Response::new(
    //         StatusCode::OK,
    //         Version::HTTP_11,
    //         Headers::new(),
    //         include_bytes!("./donuts.json").to_vec(),
    //         req,
    //     );

    //     let report = fixture.handler.process(&resp);

    //     assert_eq!(
    //         report,
    //         json!([
    //             {
    //                 "folder": "temp",
    //                 "data": []
    //             }
    //         ])
    //     );
    // }

    // #[rstest]
    // #[tokio::test]
    // async fn handler_send(fixture: Fixture) -> Result<(), Error> {
    //     let m = fixture.mock_server.mock(|when, then| {
    //         when.method("POST").path("/report");
    //         then.status(200);
    //     });
    //     let resp = fixture
    //         .handler
    //         .send(String::new(), json!([{"helloWorld": "Good Evening"}]))
    //         .unwrap()
    //         .await??;

    //     m.assert();
    //     assert_eq!(resp.status(), StatusCode::OK);

    //     Ok(())
    // }
}
