use reqwest::{ RequestBuilder, Response};
use async_trait::async_trait;
use derive_new::new;

#[derive(Debug, Clone, new)]
pub enum AuthMethod {
    BasicAuth { username: String, password: String },
    ApiKey { key: String },

}

#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub event_id: String,
    pub price: Option<String>,
}

#[derive(new, Clone, Debug)]
pub struct ProviderConfig {
    pub url: String,
    pub auth_method: AuthMethod,
}

// Define the ProviderHandler trait
#[async_trait]
pub trait ProviderHandler {
    fn config(&self) -> ProviderConfig;
    async fn authenticate(&self, request: RequestBuilder) -> RequestBuilder;
    async fn parse_response(&self, response: Response) -> Option<ProviderResponse>;
}