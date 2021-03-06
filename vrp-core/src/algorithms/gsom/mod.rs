//! Provides customized implementation of Growing Self Organizing Map.

use std::fmt::Display;

mod network;
pub use self::network::Network;

mod node;
pub use self::node::*;

mod state;
pub use self::state::*;

/// Represents an input for network.
pub trait Input: Send + Sync {
    /// Returns weights.
    fn weights(&self) -> &[f64];
}

/// Represents input data storage.
pub trait Storage: Display + Send + Sync {
    /// An input type.
    type Item: Input;

    /// Adds an input to the storage.
    fn add(&mut self, input: Self::Item);

    /// Removes and returns all data from the storage.
    fn drain(&mut self) -> Vec<Self::Item>;

    /// Returns a distance between two input weights.
    fn distance(&self, a: &[f64], b: &[f64]) -> f64;
}
