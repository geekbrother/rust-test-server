//! Adapter for Cat endpoint response
//!
//! API endpoint documentation: https://alexwohlbruck.github.io/cat-facts/docs/endpoints/facts.html
//! Implementation of the `Transformable` trait for the adapter abstraction.

use crate::facts::Transformable;
use anyhow::bail;
use serde::{Deserialize, Serialize};
use tracing::{instrument, trace};

/// Endpoint JSON response structure
#[derive(Serialize, Deserialize)]
pub struct CatFactEndpointResponse {
  pub text: String,
  #[serde(rename = "type")]
  pub animal_type: String,
  pub deleted: bool,
}

#[derive(Debug)]
pub struct Adapter;
impl Transformable for Adapter {
  /// Transforms the endpoint response into the expected fact format
  #[instrument]
  fn transform(&self, response_content: &str) -> anyhow::Result<String> {
    trace!("transforming fact content from cat endpoint response");

    let data: CatFactEndpointResponse = serde_json::from_str(response_content)?;
    if data.deleted {
      bail!("cat fact endpoint returned `deleted`=true on the fact")
    }
    if data.animal_type != "cat" {
      bail!(
        "cat fact endpoint returned fact not for a cat: {}",
        data.animal_type
      )
    }
    Ok(data.text)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Test transformation by passing input data from an API endpoint output and
  /// expect the fact should be extracted
  #[test]
  fn transforming_valid() {
    // Valid input data to test
    let fact = "some cat fact";
    let plain_text_input = serde_json::to_string(&CatFactEndpointResponse {
      text: fact.to_string(),
      animal_type: "cat".to_string(),
      deleted: false,
    })
    .unwrap();
    let transformed = Adapter.transform(&plain_text_input).unwrap();

    assert_eq!(transformed, fact, "transformed fact are not equal");
  }

  /// Test transformation by passing invalid input (not json) and expecting an error
  #[test]
  fn transforming_invalid() {
    // Invalid input data to test
    let plain_text_input = "just a plain text, not json";
    let transformed = Adapter.transform(plain_text_input);

    assert!(transformed.is_err());
  }
}
