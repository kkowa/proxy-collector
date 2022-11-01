//! Report handler module sending processed JSON documents to API endpoint

mod processor;

use async_trait::async_trait;
use kkowa_proxy::{http::{Method, Response, Uri},
                  proxy::{Flow, Handler, Reverse}};
use serde_json::json;
use tracing::{debug, error, trace};

pub use self::processor::Processor;

/// Handler for reporting processed documents to remote server.
#[derive(Debug)]
pub struct Reporter {
    /// API endpoint URL to POST docs.
    report_to: Option<Uri>,

    /// HTTP client for report.
    client: hyper::Client<hyper::client::HttpConnector>,

    /// Document processor.
    processors: Vec<Processor>,
}

impl Reporter {
    /// Create new handler.
    pub fn new(report_to: Option<Uri>, processors: Vec<Processor>) -> Self {
        Self {
            report_to,
            client: hyper::Client::default(),
            processors,
        }
    }

    /// Generate document from HTTP flow.
    fn process(&self, resp: &Response) -> serde_json::Value {
        let mut documents = Vec::with_capacity(self.processors.len());
        for processor in &self.processors {
            match processor.process(resp) {
                Some(document) => documents.push(document),
                None => debug!("document process returned nothing"),
            }
        }

        json!([
            {
                "folder": "temp",  // TODO: Each item should be categorized by something (URL or something)
                "data": serde_json::Value::Array(documents),
            }
        ])
    }

    /// Send report to `report_to` using HTTP method POST.
    fn send(
        &self,
        auth_token: String,
        report: serde_json::Value,
    ) -> Option<tokio::task::JoinHandle<Result<hyper::Response<hyper::Body>, hyper::Error>>> {
        match &self.report_to {
            Some(report_to) => {
                let http_client = self.client.clone();
                let uri = report_to.clone();
                let body = hyper::Body::from(report.to_string());

                Some(tokio::task::spawn(async move {
                    let req = hyper::Request::builder()
                        .method(Method::POST)
                        .uri(uri)
                        .header(http::header::CONTENT_TYPE, "application/json")
                        .header(
                            http::header::AUTHORIZATION,
                            format!("Bearer {token}", token = auth_token),
                        )
                        .body(body)
                        .unwrap();

                    match http_client.request(req).await {
                        Ok(resp) => Ok(resp),
                        Err(err) => {
                            error!("error occurred while sending report to destination: {err}");
                            Err(err)
                        }
                    }
                }))
            }
            None => {
                trace!("no report destination: {report}");

                None
            }
        }
    }
}

#[async_trait]
impl Handler for Reporter {
    async fn on_response(&self, flow: &Flow, resp: Response) -> Reverse {
        if let Some(credentials) = &flow.auth() {
            let report = self.process(&resp);

            // TODO: Instead of sending token, pass scheme too
            self.send(credentials.credentials().to_string(), report);
        }

        Reverse::DoNothing
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use anyhow::{Error, Result};
    use httpmock::prelude::*;
    use kkowa_proxy::http::{Headers, Method, Request, Response, StatusCode, Uri, Version};
    use rstest::*;
    use serde_json::json;

    use super::{Processor, Reporter};

    struct Fixture {
        mock_server: MockServer,
        handler: Reporter,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let mock_server = MockServer::start();
        let handler = Reporter::new(
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
        let report = fixture.handler.process(&resp);

        assert_eq!(
            report,
            json!([
                {
                    "folder": "temp",
                    "data": [
                        {
                            "extracted": {
                                "donutNames": ["Cake", "Raised", "Old Fashioned"]
                            }
                        }
                    ]
                }
            ])
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

        let report = fixture.handler.process(&resp);

        assert_eq!(
            report,
            json!([
                {
                    "folder": "temp",
                    "data": []
                }
            ])
        );
    }

    #[rstest]
    #[tokio::test]
    async fn handler_send(fixture: Fixture) -> Result<(), Error> {
        let m = fixture.mock_server.mock(|when, then| {
            when.method("POST").path("/report");
            then.status(200);
        });
        let resp = fixture
            .handler
            .send(String::new(), json!([{"helloWorld": "Good Evening"}]))
            .unwrap()
            .await??;

        m.assert();
        assert_eq!(resp.status(), StatusCode::OK);

        Ok(())
    }
}
