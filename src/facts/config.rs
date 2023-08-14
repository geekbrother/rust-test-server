use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Facts configuration structure
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FactsConfig {
  pub default: String,
  pub facts: HashMap<String, String>,
}
