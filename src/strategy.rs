//! Login strategies using the strategy pattern.
//! Each strategy implements the `LoginStrategy` trait.
//! This module provides support for GET requests, JSON POST requests, and form-encoded POST requests.

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

/// A trait representing a login attempt strategy.
/// Implementors must be Sync for parallel execution.
pub trait LoginStrategy: Sync {
    /// Attempt a login with the provided username and password.
    /// Returns true if the request returns HTTP 200, false otherwise.
    fn attempt(&self, user: &str, pass: &str) -> bool;
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

/// A strategy that sends form data via a POST request.
/// The form template should contain `%user%` and `%pass%` tokens.
pub struct FormStrategy {
    pub url: String,
    pub form_template: String,
    pub client: Client,
}

impl FormStrategy {
    /// Creates a new form POST strategy.
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

/// A dummy strategy that always fails.
pub struct DummyStrategy;

impl DummyStrategy {
    /// Creates a new dummy strategy.
    pub fn new() -> Self {
        DummyStrategy
    }
}

impl LoginStrategy for DummyStrategy {
    fn attempt(&self, _user: &str, _pass: &str) -> bool {
        false
    }
}
