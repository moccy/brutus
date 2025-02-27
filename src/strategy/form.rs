// src/strategy/form.rs

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

/// A strategy that sends form data via a POST request.
/// The form template should contain `%user%` and `%pass%` tokens.
pub struct FormStrategy {
    pub url: String,
    pub form_template: String,
    pub client: Client,
}

impl FormStrategy {
    /// Creates a new Form POST strategy.
    pub fn new(url: &str, form_template: &str) -> Self {
        FormStrategy {
            url: url.to_string(),
            form_template: form_template.to_string(),
            client: default_client(),
        }
    }
}

impl LoginStrategy for FormStrategy {
    fn attempt(&self, user: &str, pass: &str) -> bool {
        let body = self
            .form_template
            .replace("%user%", user)
            .replace("%pass%", pass);
        debug!("Form POST attempt to {} with body: {}", self.url, body);
        match self
            .client
            .post(&self.url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
        {
            Ok(resp) => {
                debug!("Form POST response: {}", resp.status());
                resp.status() == StatusCode::OK
            }
            Err(e) => {
                error!("Form POST error: {}", e);
                false
            }
        }
    }
}
