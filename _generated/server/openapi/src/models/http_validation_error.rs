/*
 * kkowa Open API
 *
 *  This page is automatically generated documentation of kkowa server's Open API.  Features are support by GraphQL. API is designed for communication between organization components to perform simple tasks with simple HTTP calls only. Advanced users may also use this API using their API keys to build their own programs.  For more information not described in this page, please contact organization or project administrators or maintainers.     
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct HttpValidationError {
    #[serde(rename = "detail", skip_serializing_if = "Option::is_none")]
    pub detail: Option<Vec<crate::models::ValidationError>>,
}

impl HttpValidationError {
    pub fn new() -> HttpValidationError {
        HttpValidationError {
            detail: None,
        }
    }
}


