// src/strategy/dummy.rs

use super::LoginStrategy;

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
