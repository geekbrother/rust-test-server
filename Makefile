clean:
	cargo clean

build:
	cargo build

release:
	cargo build --release

doc:
	cargo doc

test:
	cargo test

run:
	RUST_LOG=error cargo run

run-trace:
	RUST_LOG=trace cargo run

## Docker-related commands
docker-build: clean
	docker build -t rust-test-server .

docker-up:
	docker run -it -P --rm --name facts-server rust-test-server

# Deploy-related commands
deploy:
	@echo "Here we will run deployment script using terraform or another..."
