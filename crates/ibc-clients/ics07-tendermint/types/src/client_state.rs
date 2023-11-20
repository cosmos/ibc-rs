//! Implements the core [`ClientState`](crate::core::ics02_client::client_state::ClientState) trait
//! for the Tendermint light client.

use core::cmp::max;
use core::convert::{TryFrom, TryInto};
use core::str::FromStr;
use core::time::Duration;

use ibc_core_client_types::error::ClientError;
use ibc_core_client_types::Height;
use ibc_core_commitment_types::specs::ProofSpecs;
use ibc_core_host_types::identifiers::ChainId;
use ibc_primitives::prelude::*;
use ibc_primitives::ZERO_DURATION;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::client::v1::Height as RawHeight;
use ibc_proto::ibc::lightclients::tendermint::v1::ClientState as RawTmClientState;
use ibc_proto::Protobuf;
use prost::Message;
use tendermint::chain::id::MAX_LENGTH as MaxChainIdLen;
use tendermint::trust_threshold::TrustThresholdFraction as TendermintTrustThresholdFraction;
use tendermint_light_client_verifier::options::Options;
use tendermint_light_client_verifier::ProdVerifier;

use crate::consensus_state::ConsensusState as TmConsensusState;
use crate::error::Error;
use crate::header::Header as TmHeader;
use crate::trust_threshold::TrustThreshold;

pub const TENDERMINT_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.tendermint.v1.ClientState";

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AllowUpdate {
    pub after_expiry: bool,
    pub after_misbehaviour: bool,
}

/// Parameters needed when initializing a new `ClientState`. This type
/// exists mainly as a convenience for providing default values in
/// testing scenarios.
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

/// Contains the core implementation of the Tendermint light client
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ClientState {
    pub chain_id: ChainId,
    pub trust_level: TrustThreshold,
    pub trusting_period: Duration,
    pub unbonding_period: Duration,
    pub max_clock_drift: Duration,
    pub latest_height: Height,
    pub proof_specs: ProofSpecs,
    pub upgrade_path: Vec<String>,
    pub allow_update: AllowUpdate,
    pub frozen_height: Option<Height>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub verifier: ProdVerifier,
}

impl ClientState {
    #[allow(clippy::too_many_arguments)]
    fn new_without_validation(
        chain_id: ChainId,
        trust_level: TrustThreshold,
        trusting_period: Duration,
        unbonding_period: Duration,
        max_clock_drift: Duration,
        latest_height: Height,
        proof_specs: ProofSpecs,
        upgrade_path: Vec<String>,
        allow_update: AllowUpdate,
    ) -> Self {
        Self {
            chain_id,
            trust_level,
            trusting_period,
            unbonding_period,
            max_clock_drift,
            latest_height,
            proof_specs,
            upgrade_path,
            allow_update,
            frozen_height: None,
            verifier: ProdVerifier::default(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        chain_id: ChainId,
        trust_level: TrustThreshold,
        trusting_period: Duration,
        unbonding_period: Duration,
        max_clock_drift: Duration,
        latest_height: Height,
        proof_specs: ProofSpecs,
        upgrade_path: Vec<String>,
        allow_update: AllowUpdate,
    ) -> Result<Self, Error> {
        let client_state = Self::new_without_validation(
            chain_id,
            trust_level,
            trusting_period,
            unbonding_period,
            max_clock_drift,
            latest_height,
            proof_specs,
            upgrade_path,
            allow_update,
        );
        client_state.validate()?;
        Ok(client_state)
    }

    pub fn with_header(self, header: TmHeader) -> Result<Self, Error> {
        Ok(Self {
            latest_height: max(header.height(), self.latest_height),
            ..self
        })
    }

    pub fn with_frozen_height(self, h: Height) -> Self {
        Self {
            frozen_height: Some(h),
            ..self
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        self.chain_id.validate_length(3, MaxChainIdLen as u64)?;

        // `TrustThreshold` is guaranteed to be in the range `[0, 1)`, but a `TrustThreshold::ZERO`
        // value is invalid in this context
        if self.trust_level == TrustThreshold::ZERO {
            return Err(Error::InvalidTrustThreshold {
                reason: "ClientState trust-level cannot be zero".to_string(),
            });
        }

        TendermintTrustThresholdFraction::new(
            self.trust_level.numerator(),
            self.trust_level.denominator(),
        )
        .map_err(Error::InvalidTendermintTrustThreshold)?;

        // Basic validation of trusting period and unbonding period: each should be non-zero.
        if self.trusting_period <= Duration::new(0, 0) {
            return Err(Error::InvalidTrustThreshold {
                reason: format!(
                    "ClientState trusting period ({:?}) must be greater than zero",
                    self.trusting_period
                ),
            });
        }

        if self.unbonding_period <= Duration::new(0, 0) {
            return Err(Error::InvalidTrustThreshold {
                reason: format!(
                    "ClientState unbonding period ({:?}) must be greater than zero",
                    self.unbonding_period
                ),
            });
        }

        if self.trusting_period >= self.unbonding_period {
            return Err(Error::InvalidTrustThreshold {
                reason: format!(
                "ClientState trusting period ({:?}) must be smaller than unbonding period ({:?})", self.trusting_period, self.unbonding_period
            ),
            });
        }

        if self.max_clock_drift <= Duration::new(0, 0) {
            return Err(Error::InvalidMaxClockDrift {
                reason: "ClientState max-clock-drift must be greater than zero".to_string(),
            });
        }

        if self.latest_height.revision_number() != self.chain_id.revision_number() {
            return Err(Error::InvalidLatestHeight {
                reason: "ClientState latest-height revision number must match chain-id version"
                    .to_string(),
            });
        }

        // Disallow empty proof-specs
        if self.proof_specs.is_empty() {
            return Err(Error::Validation {
                reason: "ClientState proof-specs cannot be empty".to_string(),
            });
        }

        // `upgrade_path` itself may be empty, but if not then each key must be non-empty
        for (idx, key) in self.upgrade_path.iter().enumerate() {
            if key.trim().is_empty() {
                return Err(Error::Validation {
                    reason: format!(
                        "ClientState upgrade-path key at index {idx:?} cannot be empty"
                    ),
                });
            }
        }

        Ok(())
    }

    /// Get the refresh time to ensure the state does not expire
    pub fn refresh_time(&self) -> Option<Duration> {
        Some(2 * self.trusting_period / 3)
    }

    /// Helper method to produce a [`Options`] struct for use in
    /// Tendermint-specific light client verification.
    pub fn as_light_client_options(&self) -> Result<Options, Error> {
        Ok(Options {
            trust_threshold: self.trust_level.try_into().map_err(|e: ClientError| {
                Error::InvalidTrustThreshold {
                    reason: e.to_string(),
                }
            })?,
            trusting_period: self.trusting_period,
            clock_drift: self.max_clock_drift,
        })
    }

    pub fn chain_id(&self) -> ChainId {
        self.chain_id.clone()
    }

    pub fn is_frozen(&self) -> bool {
        self.frozen_height.is_some()
    }

    // Resets custom fields to zero values (used in `update_client`)
    pub fn zero_custom_fields(&mut self) {
        self.trusting_period = ZERO_DURATION;
        self.trust_level = TrustThreshold::ZERO;
        self.allow_update.after_expiry = false;
        self.allow_update.after_misbehaviour = false;
        self.frozen_height = None;
        self.max_clock_drift = ZERO_DURATION;
    }
}

impl Protobuf<RawTmClientState> for ClientState {}

impl TryFrom<RawTmClientState> for ClientState {
    type Error = Error;

    fn try_from(raw: RawTmClientState) -> Result<Self, Self::Error> {
        let chain_id = ChainId::from_str(raw.chain_id.as_str())?;

        let trust_level = {
            let trust_level = raw
                .trust_level
                .clone()
                .ok_or(Error::MissingTrustingPeriod)?;
            trust_level
                .try_into()
                .map_err(|e| Error::InvalidTrustThreshold {
                    reason: format!("{e}"),
                })?
        };

        let trusting_period = raw
            .trusting_period
            .ok_or(Error::MissingTrustingPeriod)?
            .try_into()
            .map_err(|_| Error::MissingTrustingPeriod)?;

        let unbonding_period = raw
            .unbonding_period
            .ok_or(Error::MissingUnbondingPeriod)?
            .try_into()
            .map_err(|_| Error::MissingUnbondingPeriod)?;

        let max_clock_drift = raw
            .max_clock_drift
            .ok_or(Error::NegativeMaxClockDrift)?
            .try_into()
            .map_err(|_| Error::NegativeMaxClockDrift)?;

        let latest_height = raw
            .latest_height
            .ok_or(Error::MissingLatestHeight)?
            .try_into()
            .map_err(|_| Error::MissingLatestHeight)?;

        // In `RawClientState`, a `frozen_height` of `0` means "not frozen".
        // See:
        // https://github.com/cosmos/ibc-go/blob/8422d0c4c35ef970539466c5bdec1cd27369bab3/modules/light-clients/07-tendermint/types/client_state.go#L74
        if raw
            .frozen_height
            .and_then(|h| Height::try_from(h).ok())
            .is_some()
        {
            return Err(Error::FrozenHeightNotAllowed);
        }

        // We use set this deprecated field just so that we can properly convert
        // it back in its raw form
        #[allow(deprecated)]
        let allow_update = AllowUpdate {
            after_expiry: raw.allow_update_after_expiry,
            after_misbehaviour: raw.allow_update_after_misbehaviour,
        };

        let client_state = Self::new_without_validation(
            chain_id,
            trust_level,
            trusting_period,
            unbonding_period,
            max_clock_drift,
            latest_height,
            raw.proof_specs.into(),
            raw.upgrade_path,
            allow_update,
        );

        Ok(client_state)
    }
}

impl From<ClientState> for RawTmClientState {
    fn from(value: ClientState) -> Self {
        #[allow(deprecated)]
        Self {
            chain_id: value.chain_id.to_string(),
            trust_level: Some(value.trust_level.into()),
            trusting_period: Some(value.trusting_period.into()),
            unbonding_period: Some(value.unbonding_period.into()),
            max_clock_drift: Some(value.max_clock_drift.into()),
            frozen_height: Some(value.frozen_height.map(|height| height.into()).unwrap_or(
                RawHeight {
                    revision_number: 0,
                    revision_height: 0,
                },
            )),
            latest_height: Some(value.latest_height.into()),
            proof_specs: value.proof_specs.into(),
            upgrade_path: value.upgrade_path,
            allow_update_after_expiry: value.allow_update.after_expiry,
            allow_update_after_misbehaviour: value.allow_update.after_misbehaviour,
        }
    }
}

impl Protobuf<Any> for ClientState {}

impl TryFrom<Any> for ClientState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        use core::ops::Deref;

        use bytes::Buf;

        fn decode_client_state<B: Buf>(buf: B) -> Result<ClientState, Error> {
            RawTmClientState::decode(buf)
                .map_err(Error::Decode)?
                .try_into()
        }

        match raw.type_url.as_str() {
            TENDERMINT_CLIENT_STATE_TYPE_URL => {
                decode_client_state(raw.value.deref()).map_err(Into::into)
            }
            _ => Err(ClientError::UnknownClientStateType {
                client_state_type: raw.type_url,
            }),
        }
    }
}

impl From<ClientState> for Any {
    fn from(client_state: ClientState) -> Self {
        Any {
            type_url: TENDERMINT_CLIENT_STATE_TYPE_URL.to_string(),
            value: Protobuf::<RawTmClientState>::encode_vec(client_state),
        }
    }
}

// `header.trusted_validator_set` was given to us by the relayer. Thus, we
// need to ensure that the relayer gave us the right set, i.e. by ensuring
// that it matches the hash we have stored on chain.
pub fn check_header_trusted_next_validator_set(
    header: &TmHeader,
    trusted_consensus_state: &TmConsensusState,
) -> Result<(), ClientError> {
    if header.trusted_next_validator_set.hash() == trusted_consensus_state.next_validators_hash {
        Ok(())
    } else {
        Err(ClientError::HeaderVerificationFailure {
            reason: "header trusted next validator set hash does not match hash stored on chain"
                .to_string(),
        })
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use tendermint_rpc::endpoint::abci_query::AbciQuery;

    use crate::serializers::tests::test_serialization_roundtrip;
    #[test]
    fn serialization_roundtrip_no_proof() {
        let json_data = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../ibc-testkit/tests/data/json/client_state.json"
        ));
        test_serialization_roundtrip::<AbciQuery>(json_data);
    }

    #[test]
    fn serialization_roundtrip_with_proof() {
        let json_data = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../ibc-testkit/tests/data/json/client_state_proof.json"
        ));
        test_serialization_roundtrip::<AbciQuery>(json_data);
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;
    use core::time::Duration;

    use ibc::core::ics02_client::height::Height;
    use ibc::core::ics23_commitment::specs::ProofSpecs;
    use ibc::core::ics24_host::identifier::ChainId;
    use ibc::core::timestamp::ZERO_DURATION;
    use ibc_proto::google::protobuf::Any;
    use ibc_proto::ibc::core::client::v1::Height as RawHeight;
    use ibc_proto::ibc::lightclients::tendermint::v1::{ClientState as RawTmClientState, Fraction};
    use ibc_proto::ics23::ProofSpec as Ics23ProofSpec;
    use ibc_testkit::utils::clients::tendermint::dummy_tendermint_header;
    use tendermint::block::Header;
    use test_log::test;

    use super::*;
    use crate::client_state::{AllowUpdate, ClientState};
    use crate::error::Error;

    impl ClientState {
        pub fn new_dummy_from_raw(frozen_height: RawHeight) -> Result<Self, Error> {
            Self::try_from(get_dummy_raw_tm_client_state(frozen_height))
        }

        pub fn new_dummy_from_header(tm_header: Header) -> Self {
            let chain_id = ChainId::from_str(tm_header.chain_id.as_str()).expect("Never fails");
            Self::new(
                chain_id.clone(),
                Default::default(),
                Duration::from_secs(64000),
                Duration::from_secs(128000),
                Duration::from_millis(3000),
                Height::new(chain_id.revision_number(), u64::from(tm_header.height))
                    .expect("Never fails"),
                Default::default(),
                Default::default(),
                AllowUpdate {
                    after_expiry: false,
                    after_misbehaviour: false,
                },
            )
            .expect("Never fails")
        }
    }

    pub fn get_dummy_raw_tm_client_state(frozen_height: RawHeight) -> RawTmClientState {
        #[allow(deprecated)]
        RawTmClientState {
            chain_id: ChainId::new("ibc-0").expect("Never fails").to_string(),
            trust_level: Some(Fraction {
                numerator: 1,
                denominator: 3,
            }),
            trusting_period: Some(Duration::from_secs(64000).into()),
            unbonding_period: Some(Duration::from_secs(128000).into()),
            max_clock_drift: Some(Duration::from_millis(3000).into()),
            latest_height: Some(Height::new(0, 10).expect("Never fails").into()),
            proof_specs: ProofSpecs::default().into(),
            upgrade_path: Default::default(),
            frozen_height: Some(frozen_height),
            allow_update_after_expiry: false,
            allow_update_after_misbehaviour: false,
        }
    }

    #[test]
    fn client_state_new() {
        // Define a "default" set of parameters to reuse throughout these tests.
        let default_params: ClientStateParams = ClientStateParams {
            id: ChainId::new("ibc-0").unwrap(),
            trust_level: TrustThreshold::ONE_THIRD,
            trusting_period: Duration::new(64000, 0),
            unbonding_period: Duration::new(128000, 0),
            max_clock_drift: Duration::new(3, 0),
            latest_height: Height::new(0, 10).expect("Never fails"),
            proof_specs: ProofSpecs::default(),
            upgrade_path: Default::default(),
            allow_update: AllowUpdate {
                after_expiry: false,
                after_misbehaviour: false,
            },
        };

        struct Test {
            name: String,
            params: ClientStateParams,
            want_pass: bool,
        }

        let tests: Vec<Test> = vec![
            Test {
                name: "Valid parameters".to_string(),
                params: default_params.clone(),
                want_pass: true,
            },
            Test {
                name: "Valid (empty) upgrade-path".to_string(),
                params: ClientStateParams {
                    upgrade_path: vec![],
                    ..default_params.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Valid upgrade-path".to_string(),
                params: ClientStateParams {
                    upgrade_path: vec!["upgrade".to_owned(), "upgradedIBCState".to_owned()],
                    ..default_params.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Valid long (50 chars) chain-id that satisfies revision_number length < `u64::MAX` length".to_string(),
                params: ClientStateParams {
                    id: ChainId::new(&format!("{}-{}", "a".repeat(29), 0)).unwrap(),
                    ..default_params.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Invalid too-long (51 chars) chain-id".to_string(),
                params: ClientStateParams {
                    id: ChainId::new(&format!("{}-{}", "a".repeat(30), 0)).unwrap(),
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid (zero) max-clock-drift period".to_string(),
                params: ClientStateParams {
                    max_clock_drift: ZERO_DURATION,
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid unbonding period".to_string(),
                params: ClientStateParams {
                    unbonding_period: ZERO_DURATION,
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid (too small) trusting period".to_string(),
                params: ClientStateParams {
                    trusting_period: ZERO_DURATION,
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid (too large) trusting period w.r.t. unbonding period".to_string(),
                params: ClientStateParams {
                    trusting_period: Duration::new(11, 0),
                    unbonding_period: Duration::new(10, 0),
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid (equal) trusting period w.r.t. unbonding period".to_string(),
                params: ClientStateParams {
                    trusting_period: Duration::new(10, 0),
                    unbonding_period: Duration::new(10, 0),
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid (zero) trusting trust threshold".to_string(),
                params: ClientStateParams {
                    trust_level: TrustThreshold::ZERO,
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid (too small) trusting trust threshold".to_string(),
                params: ClientStateParams {
                    trust_level: TrustThreshold::new(1, 4).expect("Never fails"),
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid latest height revision number (doesn't match chain)".to_string(),
                params: ClientStateParams {
                    latest_height: Height::new(1, 1).expect("Never fails"),
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid (empty) proof specs".to_string(),
                params: ClientStateParams {
                    proof_specs: ProofSpecs::from(Vec::<Ics23ProofSpec>::new()),
                    ..default_params
                },
                want_pass: false,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let p = test.params.clone();

            let cs_result = ClientState::new(
                p.id,
                p.trust_level,
                p.trusting_period,
                p.unbonding_period,
                p.max_clock_drift,
                p.latest_height,
                p.proof_specs,
                p.upgrade_path,
                p.allow_update,
            );

            assert_eq!(
                test.want_pass,
                cs_result.is_ok(),
                "ClientState::new() failed for test {}, \nmsg{:?} with error {:?}",
                test.name,
                test.params.clone(),
                cs_result.err(),
            );
        }
    }

    #[test]
    fn tm_client_state_conversions_healthy() {
        // check client state creation path from a proto type
        let tm_client_state_from_raw = ClientState::new_dummy_from_raw(RawHeight {
            revision_number: 0,
            revision_height: 0,
        });
        assert!(tm_client_state_from_raw.is_ok());

        let any_from_tm_client_state = Any::from(
            tm_client_state_from_raw
                .as_ref()
                .expect("Never fails")
                .clone(),
        );
        let tm_client_state_from_any = ClientState::try_from(any_from_tm_client_state);
        assert!(tm_client_state_from_any.is_ok());
        assert_eq!(
            tm_client_state_from_raw.expect("Never fails"),
            tm_client_state_from_any.expect("Never fails")
        );

        // check client state creation path from a tendermint header
        let tm_header = dummy_tendermint_header();
        let tm_client_state_from_header = ClientState::new_dummy_from_header(tm_header);
        let any_from_header = Any::from(tm_client_state_from_header.clone());
        let tm_client_state_from_any = ClientState::try_from(any_from_header);
        assert!(tm_client_state_from_any.is_ok());
        assert_eq!(
            tm_client_state_from_header,
            tm_client_state_from_any.expect("Never fails")
        );
    }

    #[test]
    fn tm_client_state_malformed_with_frozen_height() {
        let tm_client_state_from_raw = ClientState::new_dummy_from_raw(RawHeight {
            revision_number: 0,
            revision_height: 10,
        });
        match tm_client_state_from_raw {
            Err(Error::FrozenHeightNotAllowed) => {}
            _ => panic!("Expected to fail with FrozenHeightNotAllowed error"),
        }
    }
}
