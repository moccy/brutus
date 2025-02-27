// src/strategy.rs

use reqwest::blocking::Client;
use reqwest::StatusCode;

/// A login attempt strategy; must be Sync so it can be used in parallel.
pub trait LoginStrategy: Sync {
    /// Attempt a login using the provided username and password.
    /// Returns true if the attempt is successful (HTTP 200), false otherwise.
    fn attempt(&self, user: &str, pass: &str) -> bool;
}

/// A strategy that performs HTTP GET requests.
/// The URL template should contain `%user%` and `%pass%` tokens.
pub struct GetStrategy {
    pub url_template: String,
    pub client: Client,
}

impl GetStrategy {
    pub fn new(url_template: &str) -> Self {
        GetStrategy {
            url_template: url_template.to_string(),
            client: Client::new(),
        }
    }
}

impl LoginStrategy for GetStrategy {
    fn attempt(&self, user: &str, pass: &str) -> bool {
        let url = self
            .url_template
            .replace("%user%", user)
            .replace("%pass%", pass);
        match self.client.get(&url).send() {
            Ok(resp) => resp.status() == StatusCode::OK,
            Err(_) => false,
        }
    }
}

/// A strategy that performs HTTP POST requests.
/// The URL is used for the endpoint, and the body template should contain `%user%` and `%pass%` tokens.
pub struct PostStrategy {
    pub url: String,
    pub body_template: String,
    pub client: Client,
}

impl PostStrategy {
    pub fn new(url: &str, body_template: &str) -> Self {
        PostStrategy {
            url: url.to_string(),
            body_template: body_template.to_string(),
            client: Client::new(),
        }
    }
}

impl LoginStrategy for PostStrategy {
    fn attempt(&self, user: &str, pass: &str) -> bool {
        let body = self
            .body_template
            .replace("%user%", user)
            .replace("%pass%", pass);
        match self.client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .body(body)
            .send() {
            Ok(resp) => resp.status() == StatusCode::OK,
            Err(_) => false,
        }
    }
}

/// A dummy strategy that always fails.
pub struct DummyStrategy;

impl DummyStrategy {
    pub fn new() -> Self {
        DummyStrategy
    }
}

impl LoginStrategy for DummyStrategy {
    fn attempt(&self, _user: &str, _pass: &str) -> bool {
        false
    }
}
