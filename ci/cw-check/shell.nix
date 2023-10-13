# install nix and `nix-shell` in this directory, run `cw-check` command
{ pkgs ? import <nixpkgs> { overlays = [ (import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/refs/heads/stable.zip")) ]; } }:
let
  rust-as-on-ci = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
  cw-check = pkgs.writeShellApplication rec {
    name = "cw-check";
    runtimeInputs = [ rust-as-on-ci ];
    text = ''
        cargo build --target wasm32-unknown-unknown --no-default-features --release
        nix run github:informalsystems/cosmos.nix#cosmwasm-check ./target/wasm32-unknown-unknown/release/cw_check.wasm
    '';
  };
in
pkgs.mkShell {
  nativeBuildInputs = [ rust-as-on-ci cw-check];
}