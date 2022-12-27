use serde::{Deserialize, Serialize};

pub const JOB_TYPE: &str = "UPDATE_SEEDS";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arg {}
