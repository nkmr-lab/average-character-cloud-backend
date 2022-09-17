use std::fmt;
use std::sync::Arc;
use thiserror::Error;

#[derive(Clone, Error, Debug)]
pub struct ShareableError(pub Arc<anyhow::Error>);

impl fmt::Display for ShareableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ShareableError] {}", self.0.as_ref())
    }
}
