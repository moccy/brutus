// src/strategy/mod.rs

pub mod get;
pub mod json;
pub mod form;
pub mod dummy;

pub use get::GetStrategy;
pub use json::JsonStrategy;
pub use form::FormStrategy;
pub use dummy::DummyStrategy;

/// The trait representing a login attempt strategy. Implementors must be Sync.
pub trait LoginStrategy: Sync {
    /// Attempt a login with the provided username and password.
    /// Returns true if the response indicates success.
    fn attempt(&self, user: &str, pass: &str) -> bool;
}
