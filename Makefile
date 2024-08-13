.PHONY: help

help: ## Display help message.
	@echo "Usage: make <target>"
	@awk 'BEGIN {FS = ":.*?## "}/^[a-zA-Z_-]+:.*?## / {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

setup: ## Install development tools including nightly rustfmt, cargo-hack and cargo-release.
	rustup component add rustfmt --toolchain nightly
	cargo install cargo-hack
	cargo install cargo-release
	cargo install typos-cli taplo-cli

lint: ## Lint the code using rustfmt, clippy and whitespace lints.
	$(MAKE) fmt
	$(MAKE) clippy
	$(MAKE) lint-toml
	bash ./ci/code-quality/whitespace-lints.sh

fmt: ## Format the code using nightly rustfmt.
	cargo +nightly fmt --all --check

clippy: ## Lint the code using clippy.
	cargo clippy --all-targets --all-features
	cargo clippy --all-targets --no-default-features

lint-toml: ## Lint the TOML files using taplo.
	taplo fmt --check

typos: ## Check for typos in the code.
	typos --config $(CURDIR)/.github/typos.toml

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
	cargo test --all-targets --all-features --no-fail-fast --release
	cargo test --all-targets --no-default-features  --no-fail-fast --release

check-release: ## Check that the release build compiles.
	cargo release --workspace --no-push --no-tag \
		--exclude ibc-derive \
		--exclude ibc-primitives

release: ## Perform an actual release and publishes to crates.io.
	cargo release --workspace --no-push --no-tag --allow-branch HEAD --execute \
		--exclude ibc-derive \
		--exclude ibc-primitives
