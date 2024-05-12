use std::collections::HashMap;

use async_trait::async_trait;
use reqwest::{
    Client,
    RequestBuilder,
    Response,
};
use serde_json::Value;
use tokio::sync::RwLock;

use super::base::{
    AuthMethod,
    ProviderConfig,
    ProviderHandler,
    ProviderResponse,
};

struct TicketMasterHandler {
    config: ProviderConfig,
}

#[async_trait]
impl ProviderHandler for TicketMasterHandler {
    fn config(&self) -> ProviderConfig {
        self.config.clone()
    }

    async fn authenticate(&self, request: RequestBuilder) -> RequestBuilder {
        match &self.config.auth_method {
            AuthMethod::ApiKey {
                key,
            } => request.query(&[("apikey", key)]),
            _ => request,
        }
    }

    async fn parse_response(&self, response: Response) -> Option<ProviderResponse> {
        if response.status().is_success() {
            if let Ok(body) = response.text().await {
                if let Ok(json) = serde_json::from_str::<Value>(&body) {
                    if let Some(events) = json["_embedded"]["events"].as_array() {
                        if let Some(event) = events.first() {
                            let event_id = event["id"].as_str().unwrap_or("").to_string();
                            let price = event["priceRanges"][0]["min"]
                                .as_f64()
                                .map(|p| p.to_string());
                            return Some(ProviderResponse {
                                event_id,
                                price,
                            });
                        }
                    }
                }
            }
        }
        None
    }
}
