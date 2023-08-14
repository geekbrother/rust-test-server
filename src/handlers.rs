//! Handling axum webserver requests and tests.
//! Providing axum router by the `app()` function.

use crate::facts;
use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Json, Response},
  routing::get,
  Router,
};
use serde::{Deserialize, Serialize};
use tracing::trace;

/// API fact route JSON response format
#[derive(Deserialize, Serialize)]
pub struct FactResponse {
  fact: String,
  animal: String,
}

/// Axum router object returning router with routes, handlers and states
pub fn app(fact_resolver: facts::FactResolver) -> Router {
  trace!("configuring app router");
  Router::new()
    .route("/fact", get(facts_handler))
    .with_state(fact_resolver)
}

/// Handler for the `/fact` route. It invoking facts resolver and return the result
/// in a JSON format or an error string with the status code in case of something wrong.
async fn facts_handler(
  State(fact_resolver): State<facts::FactResolver>,
) -> Response {
  trace!("handling the fact request through facts resolver");
  match fact_resolver.get_fact().await {
    Ok((animal, fact)) => {
      (StatusCode::OK, Json(FactResponse { animal, fact })).into_response()
    }
    Err(err) => {
      (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
    }
  }
}

/// Testing axum server by having an `app` function that produces our app.
/// It makes easy to call it from tests without having to create an HTTP server itself.
/// Fully listening server test is covered by the `listening_on_the_address` test.
#[cfg(test)]
mod tests {
  use super::*;
  use crate::facts::adapters::{
    cat::CatFactEndpointResponse, dog::DogFactEndpointResponse,
  };
  use crate::facts::config::FactsConfig;
  use axum::{
    body::Body,
    http::{Request, StatusCode},
  };
  use mockito::Server;
  use std::collections::HashMap;
  use std::net::TcpListener;
  use tower::ServiceExt;

  /// Requesting our facts server for the valid animal type fact from the mocked
  /// dog facts server endpoint
  #[tokio::test]
  async fn valid_animal_type() {
    // Create a mock (mockito) API endpoint server for dog facts
    let mut server = Server::new_async().await;
    let dog_fact_url = server.url() + "/somefacts";
    let dog_fact_response = DogFactEndpointResponse {
      facts: vec!["some funny dog fact".to_string()],
      success: true,
    };
    let dog_fact_server_mock = server
      .mock("GET", "/somefacts")
      .with_body(serde_json::to_string(&dog_fact_response).unwrap())
      .create_async()
      .await;

    // Creating facts resolver
    let facts_config = FactsConfig {
      default: "dog".into(),
      facts: HashMap::from([("dog".to_string(), dog_fact_url)]),
    };
    let fact_resolver = facts::FactResolver::new(&facts_config);

    // Creating an axum router and shoot a request
    let app = app(fact_resolver);
    let response = app
      .oneshot(Request::builder().uri("/fact").body(Body::empty()).unwrap())
      .await
      .unwrap();

    // Assertions
    assert_eq!(
      response.status(),
      StatusCode::OK,
      "server response status code is not OK"
    );

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: FactResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(
      response_json.animal, "dog",
      "animal type is not as expected"
    );
    assert_eq!(
      response_json.fact, dog_fact_response.facts[0],
      "facts are different"
    );

    dog_fact_server_mock.assert();
  }

  /// Testing responses for `any` animal type
  #[tokio::test]
  async fn any_animal_type() {
    // Create a mock (mockito) API endpoint server for dog facts
    let mut dog_server = Server::new_async().await;
    let dog_fact_url = dog_server.url() + "/somefacts";
    let dog_fact_response = DogFactEndpointResponse {
      facts: vec!["some funny dog fact".to_string()],
      success: true,
    };
    let dog_fact_server_mock = dog_server
      .mock("GET", "/somefacts")
      .with_body(serde_json::to_string(&dog_fact_response).unwrap())
      .expect_at_least(1)
      .create_async()
      .await;

    // Create a mock (mockito) API endpoint server for cats facts
    let mut cat_server = Server::new_async().await;
    let cat_fact_url = cat_server.url() + "/somefacts";
    let cat_fact_response = CatFactEndpointResponse {
      text: "some funny cat fact".into(),
      animal_type: "cat".into(),
      deleted: false,
    };
    let cat_fact_server_mock = cat_server
      .mock("GET", "/somefacts")
      .with_body(serde_json::to_string(&cat_fact_response).unwrap())
      .expect_at_least(1)
      .create_async()
      .await;

    // Creating facts resolver with `any` type
    let facts_config = FactsConfig {
      default: "any".into(),
      facts: HashMap::from([
        ("dog".to_string(), dog_fact_url),
        ("cat".to_string(), cat_fact_url),
      ]),
    };
    let fact_resolver = facts::FactResolver::new(&facts_config);

    // Creating an axum router and shoot a request
    let app = app(fact_resolver);

    // Shooting our server multiple times to make sure it will request multiple
    // endpoints because `any` type is configured
    for _n in 1..10 {
      let response = app
        .clone()
        .oneshot(Request::builder().uri("/fact").body(Body::empty()).unwrap())
        .await
        .unwrap();
      assert_eq!(response.status(), StatusCode::OK);
    }

    // Mockito server `.assert()` makes sure that the mock server was requested at least
    // once (we added `.expect_at_least(1)` to the mock servers).
    // We can check `assert()` for both dog and cat servers to make sure that
    // when `any` type configured our server requested each of the API server at least
    // once expecting that `any` returns random animal facts.
    dog_fact_server_mock.assert();
    cat_fact_server_mock.assert();
  }

  // Testing an error behaviour when the animal facts server endpoint is not reachable
  // Expected behavior is returning an internal server error 500 status.
  #[tokio::test]
  async fn unresponsive_endpoint() {
    // Creating facts resolver
    let facts_config = FactsConfig {
      default: "dog".into(),
      facts: HashMap::from([(
        "dog".to_string(),
        "http://some-unreachable-url/".into(),
      )]),
    };
    let fact_resolver = facts::FactResolver::new(&facts_config);

    // Creating an axum router and shoot a request
    let app = app(fact_resolver);
    let response = app
      .oneshot(Request::builder().uri("/fact").body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
  }

  // Testing by spawning a real server that listens for requests at the address.
  // This is an integration test where we are spawning a real server and requesting
  // it by the `reqwest` client.
  #[tokio::test]
  async fn listening_on_the_address() {
    // Create a mock (mockito) API endpoint server for dog facts
    let mut server = Server::new_async().await;
    let dog_fact_url = server.url() + "/somefacts";
    let dog_fact_response = DogFactEndpointResponse {
      facts: vec!["some funny dog fact".to_string()],
      success: true,
    };
    let dog_fact_server_mock = server
      .mock("GET", "/somefacts")
      .with_body(serde_json::to_string(&dog_fact_response).unwrap())
      .create_async()
      .await;

    // Creating facts resolver
    let facts_config = FactsConfig {
      default: "dog".into(),
      facts: HashMap::from([("dog".to_string(), dog_fact_url)]),
    };
    let fact_resolver = facts::FactResolver::new(&facts_config);

    // Creating an axum server and spawn a tokio task
    let listening_address = "0.0.0.0:0"; // any available address and port
    let listener = TcpListener::bind(listening_address).unwrap();
    let current_addr = listener.local_addr().unwrap();
    let app = app(fact_resolver);
    tokio::spawn(async move {
      axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service())
        .await
        .unwrap();
    });

    // Making request to the server by the `reqwest`
    let response = reqwest::get(format!("http://{}/fact", current_addr))
      .await
      .unwrap();

    // Checking assertions
    assert_eq!(
      response.status(),
      StatusCode::OK,
      "server response status code is not OK"
    );

    let response_json: FactResponse =
      serde_json::from_str(&response.text().await.unwrap()).unwrap();

    assert_eq!(
      response_json.animal, "dog",
      "animal type is not as expected"
    );
    assert_eq!(
      response_json.fact, dog_fact_response.facts[0],
      "facts are different"
    );

    dog_fact_server_mock.assert();
  }
}
