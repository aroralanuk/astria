use std::collections::HashMap;

use reqwest::Client;
use tokio::sync::RwLock;

use crate::provider::base::{
    AuthMethod,
    ProviderHandler,
};

pub struct Oracle {
    client: Client,
    providers: Vec<Box<dyn ProviderHandler + Send + Sync>>,
    prices: RwLock<HashMap<String, String>>,
}

impl Oracle {
    pub fn new(providers: Vec<Box<dyn ProviderHandler + Send + Sync>>) -> Self {
        Oracle {
            client: Client::new(),
            providers,
            prices: RwLock::new(HashMap::new()),
        }
    }

    // Fetch prices from the providers and update the internal prices map
    pub async fn fetch_prices(&self) {
        let mut prices = HashMap::new();

        for provider in &self.providers {
            let request = self.client.get(&provider.config().url);
            let authenticated_request = provider.authenticate(request).await;
            let response = authenticated_request.send().await;

            if let Ok(response) = response {
                if let Some(provider_response) = provider.parse_response(response).await {
                    if let Some(price) = provider_response.price {
                        prices.insert(provider_response.event_id, price);
                    }
                }
            }
        }

        *self.prices.write().await = prices;
    }

    pub async fn get_prices(&self) -> HashMap<String, String> {
        self.prices.read().await.clone()
    }
}
