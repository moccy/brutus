// src/strategy/json.rs

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

/// A strategy that sends a JSON POST request.
/// It replaces `%user%` and `%pass%` tokens in the body template.
pub struct JsonStrategy {
    pub url: String,
    pub body_template: String,
    pub client: Client,
}

impl JsonStrategy {
    /// Creates a new JSON POST strategy.
    pub fn new(url: &str, body_template: &str) -> Self {
        JsonStrategy {
            url: url.to_string(),
            body_template: body_template.to_string(),
            client: default_client(),
        }
    }
}

impl LoginStrategy for JsonStrategy {
    fn attempt(&self, user: &str, pass: &str) -> bool {
        let body = self
            .body_template
            .replace("%user%", user)
            .replace("%pass%", pass);
        debug!("JSON POST attempt to {} with body: {}", self.url, body);
        match self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
        {
            Ok(resp) => {
                debug!("JSON POST response: {}", resp.status());
                resp.status() == StatusCode::OK
            }
            Err(e) => {
                error!("JSON POST error: {}", e);
                false
            }
        }
    }
}
