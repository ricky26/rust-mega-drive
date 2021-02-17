use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Metadata {
    pub entry_assembly: Option<PathBuf>,
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            entry_assembly: None,
        }
    }
}
