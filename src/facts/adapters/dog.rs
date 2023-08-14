//! Adapter for Dog endpoint response
//!
//! API documentation: https://kinduff.github.io/dog-api/
//! Implementation of the `Transformable` trait for the adapter abstraction.

use crate::facts::Transformable;
use anyhow::bail;
use serde::{Deserialize, Serialize};
use tracing::{instrument, trace};

/// Endpoint JSON response structure
#[derive(Serialize, Deserialize)]
pub struct DogFactEndpointResponse {
  pub facts: Vec<String>,
  pub success: bool,
}

#[derive(Debug)]
pub struct Adapter;
impl Transformable for Adapter {
  /// Transform the endpoint response into the expected fact format
  #[instrument]
  fn transform(&self, response_content: &str) -> anyhow::Result<String> {
    trace!("transforming fact content from the dog response");

    let data: DogFactEndpointResponse = serde_json::from_str(response_content)?;
    if !data.success {
      bail!("dog fact endpoint returned `success`=false on the fact")
    }
    if data.facts.len() > 1 {
      bail!(
        "dog fact endpoint returned more than one fact result: {}",
        data.facts.len()
      )
    }
    Ok(data.facts[0].clone())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Test transformation by passing input data from API endpoint format and
  /// expect the fact should be extracted
  #[test]
  fn transforming_valid() {
    // Valid input data to test
    let dog_fact = "some dog fact";
    let plain_text_input = serde_json::to_string(&DogFactEndpointResponse {
      facts: vec![dog_fact.to_string()],
      success: true,
    })
    .unwrap();
    let transformed = Adapter.transform(&plain_text_input).unwrap();

    assert_eq!(transformed, dog_fact, "transformed fact is not equal");
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
