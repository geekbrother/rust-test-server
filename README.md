# rust-test-server

This is a web server that returns a funny fact about animals provided by the remote
API server endpoint. The animal fact can be requested by the `GET` request to the 
`/fact` endpoint. 

The animal facts endpoints and default animal type (as well as a random animal) are 
configured by the `config.json` configuration file and transformed by using adapters 
in `src/facts/adapters`. 

The server can be easily extended for the new animal and 
endpoint by changing the configuration and by adding the corresponding adapter.

## Run it now!
To run the server run the `cargo run` command from the root of the project. 
Default listening port is `8888`. The server will built and you can query the local 
server by using `http://127.0.0.1:8888/fact` endpoint to get the JSON response back.

## Internals and configuration

### Requirements
 - **Rust** 1.71.1 [can work in a different version, but tested on the latest]
 - **Make** [optional, for using `Makefile` and `make` command]
 - **Docker** [optional, for building and running the server in the container]

### Makefile
The center of the build process is the `Makefile` which contains following build and
test commands:
  - **build:** Build the server from the source.
  - **release:** Build the release version of the server.
  - **test:** Run unit and integration tests.
  - **run:** Build and run the server.
  - **run-trace:** Build and run the server with additional tracing information.
  - **docker-build:** Build the docker image containing the server inside.
  - **docker-run:** Run the server inside the docker container.

To use the command run `make [command]`.

### Docker
There is a `Dockerfile` at the root of the project to build and run the server inside
the Docker container. To build the server use the `make docker-build` command. To run
the server inside the Docker container use `make docker-run` command.

### Manual building
You can manually build the server by using the standard `cargo build` command at the
root of the project. 

Tests can be invoked by the `cargo test` command.

### Logging
We are using `tracing` crate and simple tracing subscriber with the `env` filter to 
stdout logs, based on the `RUST_LOG` environment variable.

By defaul `make run` uses `error` logging level. To run in a tracing mode please use
`make run-trace` command or pass `RUST_LOG` environment variable with the logging level.

### Configuration
Default configuration file path is `config.json`, but the file path can be overridden 
by the `RUST_FACTS_CONFIG_FILE` environment variable. The default configuration file is
fullfilled with the real configuration data for animals and facts endpoints.

The configuration file structure is:
  - **server:** Server configuration parameters,
    - **address:** Listening address in `IP:PORT` format,
  - **animals:** List of animals and facts endpoints,
    - **default:** Default animal fact name. Can be animal name or `any` for random animal fact,
    - **facts:** List of facts endpoints in `"animal name":"API endpoint"` format.

### Adding new animals
Animals servers API differs from each other thats why first, you should implement and add
a new adapter for the animal fact server API and put it into the `src/facts/adapters/`.

The new adapter should implement the `Transformable` trait from `src/facts/mod.rs:Transformable`
and the new animal should be added to the `use_adapter` and `get_fact` in 
`src/facts/mod.rs`.

After that the endpoint for the new animal type can be provided in the configuration
file at the `animals->facts` list.

### Error Handling
Errors are handled by using `anyhow` crate, passing errors and handles them
at the most upper level. 

In case of an error the `HTTP STATUS 500` will be returned with the error message
as a plain text.

### Dependencies injection
`State` is used for dependencies injection into server handlers in `axum` router.

### Tests
The project contains folowing tests coverage:

Unit tests:
 - Animal fact adapters tests `src/facts/adapters`,
 - Configuration loader tests `src/config.rs`,
 - Axum router handler tests `src/handler.rs`.

Integration tests (with mocking remote endpoints):
 - Spawning a server and requesting it by the `reqwest` client 
 `src/handlers.rs:listening_on_the_address()`.

### Code style and linting
`rustfmt` for code formatting (see `rustfmt.toml` for formatting rules) and 
`cargo clippy` instead of `cargo check` are used.

## TODOs:
- Remove `.clone()` and get rid of `String` in struct fields by using lifetime annotations eg. `<'a>`,
- Precache `N` facts with `T` trasholding before querying the API to speed things up,
- Adding retrying in case the API fact endpoint returned an error.
