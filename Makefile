.PHONY: help

help: ## Display help message.
	@echo "Usage: make <target>"
	@awk 'BEGIN {FS = ":.*?## "}/^[a-zA-Z_-]+:.*?## / {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

install-tools: ## Install development tools including nightly rustfmt, cargo-hack and cargo-release.
	rustup component add rustfmt --toolchain nightly
	cargo install cargo-hack
	cargo install cargo-release
	cargo install typos-cli taplo-cli

lint: ## Lint the code using rustfmt, clippy and whitespace lints.
	cargo +nightly fmt --all --check
	cargo clippy --all-targets --all-features
	cargo clippy --all-targets --no-default-features
	typos --config $(CURDIR)/.github/typos.toml
	bash ./ci/code-quality/whitespace-lints.sh
	taplo fmt --check

check-features: ## Check that project compiles with all combinations of features.
	cargo hack check --workspace --feature-powerset --exclude-features default

check-docs: ## Build documentation with all features and without default features.
	cargo doc --all --all-features --release
	cargo doc --all --no-default-features --release

check-no-std: ## Check that libraries compile with `no_std` feature.
	$(MAKE) -C ./ci/no-std-check $@

check-cw: ## Check that the CosmWasm smart contract compiles.
	cd ./ci/cw-check \
	&& cargo build --target wasm32-unknown-unknown --no-default-features --release

test: ## Run tests with all features and without default features.
	cargo test --all-targets --all-features
	cargo test --all-targets --no-default-features

check-release: ## Check that the release build compiles.
	cargo release --workspace --no-push --no-tag --no-publish --exclude ibc-derive --exclude ibc-client-tendermint-cw

release: ## Perform an actual release and publishes to crates.io.
	cargo release --workspace --no-push --no-tag --exclude ibc-derive --exclude ibc-client-tendermint-cw --allow-branch HEAD --execute

build-tendermint-cw: ## Build the WASM file for the ICS-07 Tendermint light client.
	@echo "Building the WASM file for the ICS-07 Tendermint light client"
	RUSTFLAGS='-C link-arg=-s' cargo build -p ibc-client-tendermint-cw --target wasm32-unknown-unknown --release --lib --locked
	mkdir -p cw-contracts
	cp target/wasm32-unknown-unknown/release/ibc_client_tendermint_cw.wasm cw-contracts/
