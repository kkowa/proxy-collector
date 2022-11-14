//! Document processor module.

use std::{path::Path, str::FromStr};

use anyhow::{Error, Result};
use http::Method;
use json_dotpath::DotPaths;
use kkowa_proxy_lib::http::Response;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use structstruck::strike;
use tracing::{trace, warn};

type JsonValue = serde_json::Value;
type JsonDotPath = String;
type JsonPath = String;

// TODO: JSON schema generation

strike! {
    #[strikethrough[derive(Debug, Serialize, Deserialize)]]
    pub struct Processor {
        metadata: struct ProcessorMetadata {
            /// Processor identifier.
            name: String,

            /// Hostname matcher as regular expression.
            #[serde(with = "serde_regex")]
            hostname: Regex,
        },
        spec: struct ProcessorSpec {
            /// List of rules for field extraction.
            rules: Vec<SpecRule>,
        },
    }
}

impl FromStr for Processor {
    type Err = serde_yaml::Error;

    /// Create processor from raw string config.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_yaml::from_str(s)
    }
}

impl Processor {
    /// Create processor from configuration file.
    pub fn from_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let s = &std::fs::read_to_string(path)?;
        let de = Self::from_str(s)?;

        Ok(de)
    }

    /// Process given JSON document with processor's rule and generate new JSON document.
    pub fn process(&self, resp: &Response) -> Option<JsonValue> {
        let req = &resp.request;

        let hostname = req.uri.host().unwrap();
        if !self.metadata.hostname.is_match(hostname) {
            trace!(
                r#"hostname "{hostname}" does not match to regular expression `{regex}`"#,
                regex = self.metadata.hostname
            );

            return None;
        }

        let mut result = json!({});
        for rule in &self.spec.rules {
            // Check HTTP method
            let method = &req.method;
            if rule.method != method {
                trace!(
                    r#"expected method "{expected}" but got "{method}""#,
                    expected = rule.method
                );
                continue;
            }

            let path = req.uri.path();
            if !rule.path.is_match(path) {
                trace!(
                    r#"path "{path}" does not match to regular expression `{regex}`"#,
                    regex = rule.path
                );
                continue;
            }

            // Select fields from request
            if !rule.request.selectors.is_empty() {
                let maybe_document = JsonValue::from_str(&String::from_utf8_lossy(&req.payload));
                match maybe_document {
                    Ok(obj) => {
                        for selector in &rule.request.selectors {
                            selector.insert(&obj, &mut result);
                        }
                    }
                    Err(err) => warn!("can't parse JSON from request body: {err}"),
                }
            }

            // Select fields from response
            if !rule.response.selectors.is_empty() {
                let maybe_document = JsonValue::from_str(&String::from_utf8_lossy(&resp.payload));
                match maybe_document {
                    Ok(obj) => {
                        for selector in &rule.response.selectors {
                            selector.insert(&obj, &mut result);
                        }
                    }
                    Err(err) => warn!("can't parse JSON from response body: {err}"),
                }
            }
        }

        Some(result)
    }
}

strike! {
    #[strikethrough[derive(Debug, Serialize, Deserialize)]]
    struct SpecRule {
        /// Name of rule.
        name: Option<String>,

        /// Additional description of rule.
        description: Option<String>,

        /// Request method matcher.
        #[serde(with = "http_serde::method")]
        method: Method,

        /// Path component matcher for flow.
        #[serde(with = "serde_regex")]
        path: Regex,

        /// Request process rule.
        request: struct SpecRuleRequest {
            /// List of field selectors.
            selectors: Vec<Selector>,
        },

        /// Response process rule.
        response: struct SpecRuleResponse {
            /// List of field selectors.
            selectors: Vec<Selector>,
        },
    }
}

// TODO: Validation for JsonDotPath
// TODO: Validation for JsonPath and path pre-compilation
#[derive(Debug, Serialize, Deserialize)]
struct Selector {
    key: JsonDotPath,
    value: JsonPath,
}

impl Selector {
    fn insert(&self, select_from: &JsonValue, insert_to: &mut JsonValue) {
        let selector = jsonpath_lib::Compiled::compile(&self.value).unwrap();
        let new = selector.select(select_from).unwrap();
        insert_to.dot_set(&self.key, new).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use http::Uri;
    use kkowa_proxy_lib::http::{Request, Response};
    use serde_json::json;

    use super::{Processor, Selector};

    #[test]
    fn processor_from_str() {
        Processor::from_str(include_str!("donuts-processor.yaml")).unwrap();
    }

    #[test]
    fn processor_from_file() {
        Processor::from_file(
            std::path::PathBuf::from(file!())
                .parent() // Relative path to current source file
                .unwrap()
                .join("donuts-processor.yaml"),
        )
        .unwrap();
    }

    #[test]
    fn processor_process() {
        // Test with sample processor def file
        let processor = Processor::from_str(include_str!("donuts-processor.yaml")).unwrap();
        let req = Request::builder()
            .uri(Uri::from_static("http://subdomain.domain.com/donuts"))
            .build()
            .unwrap();

        let resp = Response::builder()
            .payload(include_bytes!("./donuts.json").to_vec())
            .request(req)
            .build()
            .unwrap();

        let document = processor.process(&resp).unwrap();

        assert_eq!(
            document,
            json!({
                "extracted": {
                    "donutNames": ["Cake", "Raised", "Old Fashioned"]
                }
            })
        )
    }

    #[test]
    fn selector_insert() {
        let data = serde_json::from_str(include_str!("./donuts.json")).unwrap();
        let mut document = json!({});
        let selector = Selector {
            key: "extracted.donutNames".to_string(),
            value: "$[*].name".to_string(),
        };
        selector.insert(&data, &mut document);

        assert_eq!(
            document,
            json!({
                "extracted": {
                    "donutNames": ["Cake", "Raised", "Old Fashioned"]
                }
            })
        );
    }
}
