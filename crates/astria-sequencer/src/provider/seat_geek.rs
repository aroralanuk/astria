use std::collections::HashMap;

use async_trait::async_trait;
use derive_new::new;
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

#[derive(Clone, Debug, new)]
struct SeatGeekHandler {
    config: ProviderConfig,
}

#[async_trait]
impl ProviderHandler for SeatGeekHandler {
    fn config(&self) -> ProviderConfig {
        self.config.clone()
    }

    async fn authenticate(&self, request: RequestBuilder) -> RequestBuilder {
        match &self.config.auth_method {
            AuthMethod::BasicAuth {
                username,
                password,
            } => request.basic_auth(username, Some(password)),
            _ => request,
        }
    }

    async fn parse_response(&self, response: Response) -> Option<ProviderResponse> {
        if response.status().is_success() {
            if let Ok(body) = response.text().await {
                if let Ok(json) = serde_json::from_str::<Value>(&body) {
                    let event_id = json["id"].as_str().unwrap_or("").to_string();
                    let price = json["stats"]["lowest_price"]
                        .as_str()
                        .map(|p| p.to_string());
                    return Some(ProviderResponse {
                        event_id,
                        price,
                    });
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use dotenv::dotenv;
    use reqwest::Client;

    use super::*;

    #[tokio::test]
    async fn test_seatgeek_handler_live() {
        dotenv::from_path("config/seat-geek/.env").ok();

        let username = env::var("SEATGEEK_USERNAME").expect("SEATGEEK_USERNAME not set");
        let password = env::var("SEATGEEK_PASSWORD").expect("SEATGEEK_PASSWORD not set");

        // Set up the provider configuration
        let seatgeek_config = ProviderConfig {
            url: "https://api.seatgeek.com/2/events/6109434".to_string(),
            auth_method: AuthMethod::BasicAuth {
                username,
                password,
            },
        };

        let seatgeek_handler = SeatGeekHandler {
            config: seatgeek_config,
        };

        let client = Client::new();
        let request = client.get(&seatgeek_handler.config.url);
        let authenticated_request = seatgeek_handler.authenticate(request).await;

        let response = authenticated_request.send().await.unwrap();
        let provider_response = seatgeek_handler.parse_response(response).await;

        assert!(provider_response.is_some());
        // if let Some(response) = provider_response {
        //     assert!(!response.event_id.is_empty());
        //     assert!(response.price.is_some());
        // }
    }
}
