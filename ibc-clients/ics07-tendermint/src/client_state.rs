//! This module includes trait implementations for the
//! `ibc_client_tendermint_types::ClientState` type. Implemented traits include
//! `ClientStateCommon`, `ClientStateValidation`, and `ClientStateExecution`.
//!
//! Note that this crate defines a newtype wrapper around the
//! `ibc_client_tendermint_types::ClientState` type in order to enable
//! implementing a foreign trait on a foreign type (i.e. the orphan rule in
//! Rust). As such, this module also includes some trait implementations that
//! serve to pass through traits implemented on the wrapped `ClientState` type.

use ibc_client_tendermint_types::error::Error;
use ibc_client_tendermint_types::proto::v1::ClientState as RawTmClientState;
use ibc_client_tendermint_types::ClientState as ClientStateType;
use ibc_core_client::types::error::ClientError;
use ibc_primitives::prelude::*;
use ibc_primitives::proto::{Any, Protobuf};

mod common;
mod execution;
mod misbehaviour;
mod update_client;
mod validation;

pub use common::*;
pub use execution::*;
pub use misbehaviour::*;
pub use update_client::*;
pub use validation::*;

/// Newtype wrapper around the `ClientState` type imported from the
/// `ibc-client-tendermint-types` crate. This wrapper exists so that we can
/// bypass Rust's orphan rules and implement traits from
/// `ibc::core::client::context` on the `ClientState` type.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, derive_more::From)]
pub struct ClientState(ClientStateType);

impl ClientState {
    pub fn inner(&self) -> &ClientStateType {
        &self.0
    }
}

impl Protobuf<RawTmClientState> for ClientState {}

impl TryFrom<RawTmClientState> for ClientState {
    type Error = Error;

    fn try_from(raw: RawTmClientState) -> Result<Self, Self::Error> {
        Ok(Self(ClientStateType::try_from(raw)?))
    }
}

impl From<ClientState> for RawTmClientState {
    fn from(client_state: ClientState) -> Self {
        client_state.0.into()
    }
}

impl Protobuf<Any> for ClientState {}

impl TryFrom<Any> for ClientState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        Ok(Self(ClientStateType::try_from(raw)?))
    }
}

impl From<ClientState> for Any {
    fn from(client_state: ClientState) -> Self {
        client_state.0.into()
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use ibc_client_tendermint_types::{
        AllowUpdate, ClientState as ClientStateType, TrustThreshold,
    };
    use ibc_core_client::types::Height;
    use ibc_core_commitment_types::specs::ProofSpecs;
    use ibc_core_host::types::identifiers::ChainId;

    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    pub struct ClientStateParams {
        pub id: ChainId,
        pub trust_level: TrustThreshold,
        pub trusting_period: Duration,
        pub unbonding_period: Duration,
        pub max_clock_drift: Duration,
        pub latest_height: Height,
        pub proof_specs: ProofSpecs,
        pub upgrade_path: Vec<String>,
        pub allow_update: AllowUpdate,
    }

    #[test]
    fn client_state_verify_height() {
        // Define a "default" set of parameters to reuse throughout these tests.
        let default_params: ClientStateParams = ClientStateParams {
            id: ChainId::new("ibc-1").unwrap(),
            trust_level: TrustThreshold::ONE_THIRD,
            trusting_period: Duration::new(64000, 0),
            unbonding_period: Duration::new(128_000, 0),
            max_clock_drift: Duration::new(3, 0),
            latest_height: Height::new(1, 10).expect("Never fails"),
            proof_specs: ProofSpecs::cosmos(),
            upgrade_path: Vec::new(),
            allow_update: AllowUpdate {
                after_expiry: false,
                after_misbehaviour: false,
            },
        };

        struct Test {
            name: String,
            height: Height,
            setup: Option<Box<dyn FnOnce(ClientState) -> ClientState>>,
            want_pass: bool,
        }

        let tests = vec![
            Test {
                name: "Successful height verification".to_string(),
                height: Height::new(1, 8).expect("Never fails"),
                setup: None,
                want_pass: true,
            },
            Test {
                name: "Invalid (too large)  client height".to_string(),
                height: Height::new(1, 12).expect("Never fails"),
                setup: None,
                want_pass: false,
            },
        ];

        for test in tests {
            let p = default_params.clone();
            let client_state = ClientStateType::new(
                p.id,
                p.trust_level,
                p.trusting_period,
                p.unbonding_period,
                p.max_clock_drift,
                p.latest_height,
                p.proof_specs,
                p.upgrade_path,
                p.allow_update,
            )
            .expect("Never fails");
            let client_state = match test.setup {
                Some(setup) => (setup)(ClientState(client_state)),
                _ => ClientState(client_state),
            };
            let res = validate_proof_height(client_state.inner(), test.height);

            assert_eq!(
                test.want_pass,
                res.is_ok(),
                "ClientState::validate_proof_height() failed for test {}, \nmsg{:?} with error {:?}",
                test.name,
                test.height,
                res.err(),
            );
        }
    }
}
