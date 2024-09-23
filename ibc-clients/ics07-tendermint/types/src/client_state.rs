//! Contains the implementation of the Tendermint `ClientState` domain type.

use core::cmp::max;
use core::str::FromStr;
use core::time::Duration;

use ibc_core_client_types::proto::v1::Height as RawHeight;
use ibc_core_client_types::Height;
use ibc_core_commitment_types::specs::ProofSpecs;
use ibc_core_host_types::error::DecodingError;
use ibc_core_host_types::identifiers::ChainId;
use ibc_primitives::prelude::*;
use ibc_primitives::ZERO_DURATION;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::lightclients::tendermint::v1::ClientState as RawTmClientState;
use ibc_proto::Protobuf;
use tendermint::chain::id::MAX_LENGTH as MaxChainIdLen;
use tendermint::trust_threshold::TrustThresholdFraction as TendermintTrustThresholdFraction;
use tendermint_light_client_verifier::options::Options;

use crate::error::TendermintClientError;
use crate::header::Header as TmHeader;
use crate::trust_threshold::TrustThreshold;

pub const TENDERMINT_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.tendermint.v1.ClientState";

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AllowUpdate {
    pub after_expiry: bool,
    pub after_misbehaviour: bool,
}

/// Defines data structure for Tendermint client state.
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
        frozen_height: Option<Height>,
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
            frozen_height,
        }
    }

    /// Constructs a new Tendermint `ClientState` by given parameters and checks
    /// if the parameters are valid.
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
    ) -> Result<Self, TendermintClientError> {
        let client_state = Self::new_without_validation(
            chain_id,
            trust_level,
            trusting_period,
            unbonding_period,
            max_clock_drift,
            latest_height,
            proof_specs,
            upgrade_path,
            None, // New valid client must not be frozen.
            allow_update,
        );
        client_state.validate()?;
        Ok(client_state)
    }

    pub fn with_header(self, header: TmHeader) -> Result<Self, TendermintClientError> {
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

    pub fn validate(&self) -> Result<(), TendermintClientError> {
        self.chain_id.validate_length(3, MaxChainIdLen as u64)?;

        // `TrustThreshold` is guaranteed to be in the range `[0, 1)`, but a `TrustThreshold::ZERO`
        // value is invalid in this context
        if self.trust_level == TrustThreshold::ZERO {
            return Err(TendermintClientError::InvalidTrustThreshold {
                description: "ClientState trust-level cannot be zero".to_string(),
            });
        }

        TendermintTrustThresholdFraction::new(
            self.trust_level.numerator(),
            self.trust_level.denominator(),
        )
        .map_err(|_| TendermintClientError::InvalidTrustThreshold {
            description: format!(
                "invalid Tendermint trust threshold: {:?}/{:?}",
                self.trust_level.numerator(),
                self.trust_level.denominator()
            ),
        })?;

        // Basic validation of trusting period and unbonding period: each should be non-zero.
        if self.trusting_period <= Duration::new(0, 0) {
            return Err(TendermintClientError::InvalidTrustThreshold {
                description: format!(
                    "ClientState trusting period ({:?}) must be greater than zero",
                    self.trusting_period
                ),
            });
        }

        if self.unbonding_period <= Duration::new(0, 0) {
            return Err(TendermintClientError::InvalidTrustThreshold {
                description: format!(
                    "ClientState unbonding period ({:?}) must be greater than zero",
                    self.unbonding_period
                ),
            });
        }

        if self.trusting_period >= self.unbonding_period {
            return Err(TendermintClientError::InvalidTrustThreshold {
                description: format!(
                "ClientState trusting period ({:?}) must be smaller than unbonding period ({:?})", self.trusting_period, self.unbonding_period
            ),
            });
        }

        if self.max_clock_drift <= Duration::new(0, 0) {
            return Err(TendermintClientError::InvalidMaxClockDrift);
        }

        if self.latest_height.revision_number() != self.chain_id.revision_number() {
            return Err(TendermintClientError::MismatchedRevisionHeights {
                expected: self.chain_id.revision_number(),
                actual: self.latest_height.revision_number(),
            });
        }

        // Sanity checks on client proof specs
        self.proof_specs.validate()?;

        // `upgrade_path` itself may be empty, but if not then each key must be non-empty
        for key in self.upgrade_path.iter() {
            if key.trim().is_empty() {
                return Err(TendermintClientError::MissingUpgradePathKey);
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
    pub fn as_light_client_options(&self) -> Result<Options, TendermintClientError> {
        Ok(Options {
            trust_threshold: self.trust_level.try_into()?,
            trusting_period: self.trusting_period,
            clock_drift: self.max_clock_drift,
        })
    }

    pub fn chain_id(&self) -> &ChainId {
        &self.chain_id
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
    type Error = DecodingError;

    fn try_from(raw: RawTmClientState) -> Result<Self, Self::Error> {
        let chain_id = ChainId::from_str(raw.chain_id.as_str())?;

        let trust_level = {
            let trust_level = raw.trust_level.ok_or(DecodingError::missing_raw_data(
                "tm client state trust level",
            ))?;
            trust_level.try_into()?
        };

        let trusting_period = raw
            .trusting_period
            .ok_or(DecodingError::missing_raw_data(
                "tm client state trusting period",
            ))?
            .try_into()
            .map_err(|d| {
                DecodingError::invalid_raw_data(format!("tm client state trusting period: {d:?}"))
            })?;

        let unbonding_period = raw
            .unbonding_period
            .ok_or(DecodingError::missing_raw_data(
                "tm client state unbonding period",
            ))?
            .try_into()
            .map_err(|d| {
                DecodingError::invalid_raw_data(format!("tm client state unbonding period: {d:?}"))
            })?;

        let max_clock_drift = raw
            .max_clock_drift
            .ok_or(DecodingError::missing_raw_data(
                "tm client state max clock drift",
            ))?
            .try_into()
            .map_err(|d| {
                DecodingError::invalid_raw_data(format!("tm client state max clock drift: {d:?}"))
            })?;

        let latest_height = raw
            .latest_height
            .ok_or(DecodingError::missing_raw_data(
                "tm client state latest height",
            ))?
            .try_into()?;

        let proof_specs = raw.proof_specs.try_into()?;

        // NOTE: In `RawClientState`, a `frozen_height` of `0` means "not
        // frozen". See:
        // https://github.com/cosmos/ibc-go/blob/8422d0c4c35ef970539466c5bdec1cd27369bab3/modules/light-clients/07-tendermint/types/client_state.go#L74
        let frozen_height = Height::try_from(raw.frozen_height.ok_or(
            DecodingError::missing_raw_data("tm client state frozen height"),
        )?)
        .ok();

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
            proof_specs,
            raw.upgrade_path,
            frozen_height,
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
            // NOTE: The protobuf encoded `frozen_height` of an active client
            // must be set to `0` so that `ibc-go` driven chains can properly
            // decode the `ClientState` value. In `RawClientState`, a
            // `frozen_height` of `0` means "not frozen". See:
            // https://github.com/cosmos/ibc-go/blob/8422d0c4c35ef970539466c5bdec1cd27369bab3/modules/light-clients/07-tendermint/types/client_state.go#L74
            frozen_height: Some(value.frozen_height.map(Into::into).unwrap_or(RawHeight {
                revision_number: 0,
                revision_height: 0,
            })),
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
    type Error = DecodingError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        if let TENDERMINT_CLIENT_STATE_TYPE_URL = raw.type_url.as_str() {
            Protobuf::<RawTmClientState>::decode(raw.value.as_ref()).map_err(Into::into)
        } else {
            Err(DecodingError::MismatchedResourceName {
                expected: TENDERMINT_CLIENT_STATE_TYPE_URL.to_string(),
                actual: raw.type_url,
            })
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

#[cfg(test)]
mod tests {
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
    fn client_state_new() {
        // Define a "default" set of parameters to reuse throughout these tests.
        let default_params: ClientStateParams = ClientStateParams {
            id: ChainId::new("ibc-0").unwrap(),
            trust_level: TrustThreshold::ONE_THIRD,
            trusting_period: Duration::new(64000, 0),
            unbonding_period: Duration::new(128_000, 0),
            max_clock_drift: Duration::new(3, 0),
            latest_height: Height::new(0, 10).expect("Never fails"),
            proof_specs: ProofSpecs::cosmos(),
            upgrade_path: Vec::new(),
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
        ]
        .into_iter()
        .collect();

        for test in tests {
            let p = test.params.clone();

            let cs_result: Result<ClientState, TendermintClientError> = ClientState::new(
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
}
