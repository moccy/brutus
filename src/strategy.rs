use reqwest::blocking::Client;
use reqwest::StatusCode;

/// A trait representing a login attempt strategy. Must be Sync for parallel execution.
pub trait LoginStrategy: Sync {
    /// Attempt a login with the given username and password.
    /// Returns true if the attempt is successful (HTTP 200), false otherwise.
    fn attempt(&self, user: &str, pass: &str) -> bool;
}

/// A GET strategy which replaces `%user%` and `%pass%` tokens in the URL.
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

/// A strategy that sends a JSON POST request.
/// It replaces tokens in a body template and sets the Content-Type to application/json.
pub struct JsonStrategy {
    pub url: String,
    pub body_template: String,
    pub client: Client,
}

impl JsonStrategy {
    pub fn new(url: &str, body_template: &str) -> Self {
        JsonStrategy {
            url: url.to_string(),
            body_template: body_template.to_string(),
            client: Client::new(),
        }
    }
}

impl LoginStrategy for JsonStrategy {
    fn attempt(&self, user: &str, pass: &str) -> bool {
        let body = self
            .body_template
            .replace("%user%", user)
            .replace("%pass%", pass);
        match self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
        {
            Ok(resp) => resp.status() == StatusCode::OK,
            Err(_) => false,
        }
    }
}

/// A strategy that sends form data (application/x-www-form-urlencoded) via POST.
/// The form template should contain `%user%` and `%pass%` tokens.
pub struct FormStrategy {
    pub url: String,
    pub form_template: String,
    pub client: Client,
}

impl FormStrategy {
    pub fn new(url: &str, form_template: &str) -> Self {
        FormStrategy {
            url: url.to_string(),
            form_template: form_template.to_string(),
            client: Client::new(),
        }
    }
}

impl LoginStrategy for FormStrategy {
    fn attempt(&self, user: &str, pass: &str) -> bool {
        let body = self
            .form_template
            .replace("%user%", user)
            .replace("%pass%", pass);
        match self
            .client
            .post(&self.url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
        {
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
