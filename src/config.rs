//! Configuration file loader
//!
//! Loads and parses the JSON configuration file into configuration structures

use crate::facts::config::FactsConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use tracing::{instrument, trace};

/// Service configuration data structure
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct ConfigData {
  pub server: ServerConfig,
  pub animals: FactsConfig,
}

/// API server configuration parameters
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct ServerConfig {
  pub address: String,
}

impl ConfigData {
  #[instrument]
  pub fn new(filename: &str) -> anyhow::Result<Self> {
    trace!("loading json configuration file");
    let contents = fs::read_to_string(filename)?;
    let data: ConfigData = serde_json::from_str(&contents)?;

    Ok(data)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashMap;
  use std::io::Write;
  use tempfile::NamedTempFile;

  /// Testing configuration loader by making a valid configuration JSON and
  /// writing it to a temporary file. Then create a new configuration instance and
  /// comparing the valid configuration with the derived one.
  #[test]
  fn valid_config() {
    // Create a valid configuration and serialize it into a plain text
    let valid_config_data = ConfigData {
      server: ServerConfig {
        address: "0.0.0.0:8888".into(),
      },
      animals: FactsConfig {
        default: "dog".into(),
        facts: HashMap::from([(
          "dog".to_string(),
          "http://some-dog-endpoint".to_string(),
        )]),
      },
    };
    let plain_text_config = serde_json::to_string(&valid_config_data).unwrap();

    // Write the configuration into the temporary file
    // `NamedTempFile` handling cleanup on the destruction so we can omit removing the
    // temporary file
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "{}", plain_text_config).unwrap();

    let path = file.path().to_str().unwrap();
    let config_data = ConfigData::new(path).unwrap();

    assert!(
      valid_config_data == config_data,
      "config data structs are not equal"
    );
  }

  /// Testing configuration loader error behavior when providing wrong formatted
  /// configuration file
  #[test]
  fn invalid_config() {
    // Create an invalid formatted configuration
    let invalid_config_data = "this is not a valid json configuration";

    // Write the configuration into the temporary file
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "{}", invalid_config_data).unwrap();

    let path = file.path().to_str().unwrap();
    let config_data = ConfigData::new(path);

    assert!(config_data.is_err())
  }

  /// Testing configuration loader error behavior when a configuration
  /// file does not exist
  #[test]
  fn config_file_not_exist() {
    let config_data = ConfigData::new("blah_blah_blah.json");
    assert!(config_data.is_err())
  }
}
