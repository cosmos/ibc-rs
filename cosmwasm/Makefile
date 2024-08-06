.PHONY: help

help: ## Display help message.
	@echo "Usage: make <target>"
	@awk 'BEGIN {FS = ":.*?## "}/^[a-zA-Z_-]+:.*?## / {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

install-tools: ## Install development tools including nightly rustfmt, cargo-hack and cargo-release.
	rustup target add wasm32-unknown-unknown

lint: ## Lint the code using rustfmt, clippy and whitespace lints.
	cargo +nightly fmt --all --check
	cargo clippy --all-targets --all-features
	cargo clippy --all-targets --no-default-features

check-docs: ## Build documentation with all features and without default features.
	cargo doc --all --all-features --release

test: ## Run tests with all features and without default features.
	cargo test --all-targets --all-features --release

check-release: ## Check that the release build compiles.
	cargo release --workspace --no-push --no-tag \
		--exclude ibc-client-tendermint-cw

release: ## Perform an actual release and publishes to crates.io.
	cargo release --workspace --no-push --no-tag --allow-branch HEAD --execute \
		--exclude ibc-client-tendermint-cw

build-tendermint-cw: ## Build the WASM file for the ICS-07 Tendermint light client.
	@echo "Building the WASM file for the ICS-07 Tendermint light client"
	    RUSTFLAGS='-C link-arg=-s' cargo build -p ibc-client-tendermint-cw --target wasm32-unknown-unknown --release --lib --locked && \
	    mkdir -p cw-contracts && \
	    cp target/wasm32-unknown-unknown/release/ibc_client_tendermint_cw.wasm cw-contracts/
