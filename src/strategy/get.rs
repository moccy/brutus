// src/strategy/get.rs

use super::LoginStrategy;
use log::{debug, error};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use std::time::Duration;

/// Returns a reqwest blocking client with a 10-second timeout.
fn default_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build HTTP client")
}

/// A GET strategy which replaces `%user%` and `%pass%` tokens in the URL.
pub struct GetStrategy {
    pub url_template: String,
    pub client: Client,
}

impl GetStrategy {
    /// Creates a new GET strategy.
    pub fn new(url_template: &str) -> Self {
        GetStrategy {
            url_template: url_template.to_string(),
            client: default_client(),
        }
    }
}

impl LoginStrategy for GetStrategy {
    fn attempt(&self, user: &str, pass: &str) -> bool {
        let url = self
            .url_template
            .replace("%user%", user)
            .replace("%pass%", pass);
        debug!("GET attempt: {}", url);
        match self.client.get(&url).send() {
            Ok(resp) => {
                debug!("GET response: {}", resp.status());
                resp.status() == StatusCode::OK
            }
            Err(e) => {
                error!("GET error: {}", e);
                false
            }
        }
    }
}
