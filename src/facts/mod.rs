//! Facts resolver
//!
//! Resolving facts by querying the fact API endpoint from the configuration
//! depending on the fact type needed and transform the result into our server's
//! result response format by using the adapters provided in the `adapters` folder.
//!
//! This module is tested at the upper level in the `handlers.rs` to leverage the
//! testing repetition

use self::config::FactsConfig;
use anyhow::bail;
use rand::seq::SliceRandom;
use tracing::{instrument, trace};

pub mod adapters;
pub mod config;

/// Fact transform adapters should implement the Transformable trait
pub trait Transformable {
  fn transform(&self, response_content: &str) -> anyhow::Result<String>;
}

#[derive(Debug, Clone)]
pub struct FactResolver {
  config: FactsConfig,
}

impl FactResolver {
  #[instrument]
  pub fn new(config: &config::FactsConfig) -> Self {
    trace!("constructing new facts resolver");
    FactResolver {
      // We can possibly remove this clonning by using a lifitime attribute <'a>
      config: config.clone(),
    }
  }

  /// Making a request to the API endpoint by shooting a GET reqwest and returning
  /// the response body as a string.
  #[instrument]
  async fn request_api(&self, endpoint: &str) -> anyhow::Result<String> {
    trace!("requesting dogs endpoint: {}", endpoint);
    let content = reqwest::get(endpoint).await?.text().await?;
    Ok(content)
  }

  /// Calling the transform method of the given adapter
  fn transfrom(
    &self,
    adapter: &impl Transformable,
    input: &str,
  ) -> anyhow::Result<String> {
    trace!("transforming imput using adapter");
    adapter.transform(input)
  }

  /// Choose which adapter to use based on the animal name and check if that adapter is
  /// present
  #[instrument]
  fn use_adapter(
    &self,
    animal_type: &str,
    input: &str,
  ) -> anyhow::Result<String> {
    trace!("choosing adapter based on the animal type: {}", animal_type);
    match animal_type {
      "dog" => self.transfrom(&adapters::dog::Adapter, input),
      "cat" => self.transfrom(&adapters::cat::Adapter, input),
      _ => bail!("invalid animal type: {}", animal_type),
    }
  }

  /// Getting the fact from the remote endpoint for the animal based
  /// on the configuration parameters.
  /// Result is a tuple with the animal type and the animal fact text.
  #[instrument]
  pub async fn get_fact(&self) -> anyhow::Result<(String, String)> {
    trace!("getting the animal fact");
    let animal_fact = if self.config.default == "any" {
      vec!["dog".to_string(), "cat".to_string()]
        .choose(&mut rand::thread_rng())
        .unwrap()
        .clone()
    } else {
      self.config.default.clone()
    };

    // Check if the configuration contains the animal type
    if !self.config.facts.contains_key(&animal_fact) {
      bail!(
        "the animal type does not exist in the configuration file: {}",
        animal_fact
      );
    }

    let endpoint_api = self.config.facts[&animal_fact].clone();
    let response = self.request_api(&endpoint_api).await?;
    let fact_text = self.use_adapter(&animal_fact, &response)?;

    Ok((animal_fact, fact_text))
  }
}
