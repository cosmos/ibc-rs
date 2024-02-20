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
use ibc_client_tendermint_types::{
    client_type as tm_client_type, ClientState as ClientStateType,
    ConsensusState as ConsensusStateType, Header as TmHeader, Misbehaviour as TmMisbehaviour,
    TENDERMINT_HEADER_TYPE_URL, TENDERMINT_MISBEHAVIOUR_TYPE_URL,
};
use ibc_core_client::context::client_state::{
    ClientStateCommon, ClientStateExecution, ClientStateValidation,
};
use ibc_core_client::context::consensus_state::ConsensusState;
use ibc_core_client::context::{ClientExecutionContext, ClientValidationContext};
use ibc_core_client::types::error::{ClientError, UpgradeClientError};
use ibc_core_client::types::{Height, Status};
use ibc_core_commitment_types::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use ibc_core_commitment_types::merkle::{apply_prefix, MerkleProof};
use ibc_core_host::types::identifiers::{ClientId, ClientType};
use ibc_core_host::types::path::{
    ClientConsensusStatePath, ClientStatePath, Path, UpgradeClientPath,
};
use ibc_core_host::ExecutionContext;
use ibc_primitives::prelude::*;
use ibc_primitives::proto::{Any, Protobuf};
use ibc_primitives::ToVec;

use super::consensus_state::ConsensusState as TmConsensusState;
use crate::context::{
    CommonContext, ExecutionContext as TmExecutionContext, ValidationContext as TmValidationContext,
};

mod misbehaviour;
mod update_client;

/// Newtype wrapper around the `ClientState` type imported from the
/// `ibc-client-tendermint-types` crate. This wrapper exists so that we can
/// bypass Rust's orphan rules and implement traits from
/// `ibc::core::client::context` on the `ClientState` type.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ClientState(ClientStateType);

impl ClientState {
    pub fn inner(&self) -> &ClientStateType {
        &self.0
    }
}

impl From<ClientStateType> for ClientState {
    fn from(client_state: ClientStateType) -> Self {
        Self(client_state)
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

impl ClientStateCommon for ClientState {
    fn verify_consensus_state(&self, consensus_state: Any) -> Result<(), ClientError> {
        let tm_consensus_state = TmConsensusState::try_from(consensus_state)?;
        if tm_consensus_state.root().is_empty() {
            return Err(ClientError::Other {
                description: "empty commitment root".into(),
            });
        };

        Ok(())
    }

    fn client_type(&self) -> ClientType {
        tm_client_type()
    }

    fn latest_height(&self) -> Height {
        self.0.latest_height
    }

    fn validate_proof_height(&self, proof_height: Height) -> Result<(), ClientError> {
        if self.latest_height() < proof_height {
            return Err(ClientError::InvalidProofHeight {
                latest_height: self.latest_height(),
                proof_height,
            });
        }
        Ok(())
    }

    /// Perform client-specific verifications and check all data in the new
    /// client state to be the same across all valid Tendermint clients for the
    /// new chain.
    ///
    /// You can learn more about how to upgrade IBC-connected SDK chains in
    /// [this](https://ibc.cosmos.network/main/ibc/upgrades/quick-guide.html)
    /// guide
    fn verify_upgrade_client(
        &self,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
        proof_upgrade_client: CommitmentProofBytes,
        proof_upgrade_consensus_state: CommitmentProofBytes,
        root: &CommitmentRoot,
    ) -> Result<(), ClientError> {
        // Make sure that the client type is of Tendermint type `ClientState`
        let upgraded_tm_client_state = Self::try_from(upgraded_client_state.clone())?;

        // Make sure that the consensus type is of Tendermint type `ConsensusState`
        TmConsensusState::try_from(upgraded_consensus_state.clone())?;

        // Make sure the latest height of the current client is not greater then
        // the upgrade height This condition checks both the revision number and
        // the height
        if self.latest_height() >= upgraded_tm_client_state.0.latest_height {
            return Err(UpgradeClientError::LowUpgradeHeight {
                upgraded_height: self.latest_height(),
                client_height: upgraded_tm_client_state.0.latest_height,
            })?;
        }

        // Check to see if the upgrade path is set
        let mut upgrade_path = self.0.upgrade_path.clone();
        if upgrade_path.pop().is_none() {
            return Err(ClientError::ClientSpecific {
                description: "cannot upgrade client as no upgrade path has been set".to_string(),
            });
        };

        let upgrade_path_prefix = CommitmentPrefix::try_from(upgrade_path[0].clone().into_bytes())
            .map_err(ClientError::InvalidCommitmentProof)?;

        let last_height = self.latest_height().revision_height();

        // Verify the proof of the upgraded client state
        self.verify_membership(
            &upgrade_path_prefix,
            &proof_upgrade_client,
            root,
            Path::UpgradeClient(UpgradeClientPath::UpgradedClientState(last_height)),
            upgraded_client_state.to_vec(),
        )?;

        // Verify the proof of the upgraded consensus state
        self.verify_membership(
            &upgrade_path_prefix,
            &proof_upgrade_consensus_state,
            root,
            Path::UpgradeClient(UpgradeClientPath::UpgradedClientConsensusState(last_height)),
            upgraded_consensus_state.to_vec(),
        )?;

        Ok(())
    }

    fn verify_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
        value: Vec<u8>,
    ) -> Result<(), ClientError> {
        let merkle_path = apply_prefix(prefix, vec![path.to_string()]);
        let merkle_proof =
            MerkleProof::try_from(proof).map_err(ClientError::InvalidCommitmentProof)?;

        merkle_proof
            .verify_membership(
                &self.0.proof_specs,
                root.clone().into(),
                merkle_path,
                value,
                0,
            )
            .map_err(ClientError::Ics23Verification)
    }

    fn verify_non_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
    ) -> Result<(), ClientError> {
        let merkle_path = apply_prefix(prefix, vec![path.to_string()]);
        let merkle_proof =
            MerkleProof::try_from(proof).map_err(ClientError::InvalidCommitmentProof)?;

        merkle_proof
            .verify_non_membership(&self.0.proof_specs, root.clone().into(), merkle_path)
            .map_err(ClientError::Ics23Verification)
    }
}

impl<V> ClientStateValidation<V> for ClientState
where
    V: ClientValidationContext + TmValidationContext,
    V::AnyConsensusState: TryInto<TmConsensusState>,
    ClientError: From<<V::AnyConsensusState as TryInto<TmConsensusState>>::Error>,
{
    fn verify_client_message(
        &self,
        ctx: &V,
        client_id: &ClientId,
        client_message: Any,
    ) -> Result<(), ClientError> {
        match client_message.type_url.as_str() {
            TENDERMINT_HEADER_TYPE_URL => {
                let header = TmHeader::try_from(client_message)?;
                self.verify_header(ctx, client_id, header)
            }
            TENDERMINT_MISBEHAVIOUR_TYPE_URL => {
                let misbehaviour = TmMisbehaviour::try_from(client_message)?;
                self.verify_misbehaviour(ctx, client_id, misbehaviour)
            }
            _ => Err(ClientError::InvalidUpdateClientMessage),
        }
    }

    /// Namely the following cases are covered:
    ///
    /// 1 - fork:
    /// Assumes at least one consensus state before the fork point exists. Let
    /// existing consensus states on chain B be: [Sn,.., Sf, Sf-1, S0] with
    /// `Sf-1` being the most recent state before fork. Chain A is queried for a
    /// header `Hf'` at `Sf.height` and if it is different than the `Hf` in the
    /// event for the client update (the one that has generated `Sf` on chain),
    /// then the two headers are included in the evidence and submitted. Note
    /// that in this case the headers are different but have the same height.
    ///
    /// 2 - BFT time violation for unavailable header (a.k.a. Future Lunatic
    /// Attack or FLA):
    /// Some header with a height that is higher than the latest height on A has
    /// been accepted and a consensus state was created on B. Note that this
    /// implies that the timestamp of this header must be within the
    /// `clock_drift` of the client. Assume the client on B has been updated
    /// with `h2`(not present on/ produced by chain A) and it has a timestamp of
    /// `t2` that is at most `clock_drift` in the future. Then the latest header
    /// from A is fetched, let it be `h1`, with a timestamp of `t1`. If `t1 >=
    /// t2` then evidence of misbehavior is submitted to A.
    ///
    /// 3 - BFT time violation for existing headers:
    /// Ensure that consensus state times are monotonically increasing with
    /// height.
    fn check_for_misbehaviour(
        &self,
        ctx: &V,
        client_id: &ClientId,
        client_message: Any,
    ) -> Result<bool, ClientError> {
        match client_message.type_url.as_str() {
            TENDERMINT_HEADER_TYPE_URL => {
                let header = TmHeader::try_from(client_message)?;
                self.check_for_misbehaviour_update_client(ctx, client_id, header)
            }
            TENDERMINT_MISBEHAVIOUR_TYPE_URL => {
                let misbehaviour = TmMisbehaviour::try_from(client_message)?;
                self.check_for_misbehaviour_misbehavior(&misbehaviour)
            }
            _ => Err(ClientError::InvalidUpdateClientMessage),
        }
    }

    fn status(&self, ctx: &V, client_id: &ClientId) -> Result<Status, ClientError> {
        if self.0.is_frozen() {
            return Ok(Status::Frozen);
        }

        let latest_consensus_state: TmConsensusState = {
            let any_latest_consensus_state =
                match ctx.consensus_state(&ClientConsensusStatePath::new(
                    client_id.clone(),
                    self.0.latest_height.revision_number(),
                    self.0.latest_height.revision_height(),
                )) {
                    Ok(cs) => cs,
                    // if the client state does not have an associated consensus state for its latest height
                    // then it must be expired
                    Err(_) => return Ok(Status::Expired),
                };

            any_latest_consensus_state.try_into()?
        };

        // Note: if the `duration_since()` is `None`, indicating that the latest
        // consensus state is in the future, then we don't consider the client
        // to be expired.
        let now = ctx.host_timestamp()?;
        if let Some(elapsed_since_latest_consensus_state) =
            now.duration_since(&latest_consensus_state.timestamp().into())
        {
            if elapsed_since_latest_consensus_state > self.0.trusting_period {
                return Ok(Status::Expired);
            }
        }

        Ok(Status::Active)
    }
}

impl<E> ClientStateExecution<E> for ClientState
where
    E: TmExecutionContext + ExecutionContext,
    <E as ClientExecutionContext>::AnyClientState: From<ClientState>,
    <E as ClientExecutionContext>::AnyConsensusState: From<TmConsensusState>,
{
    fn initialise(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        consensus_state: Any,
    ) -> Result<(), ClientError> {
        let host_timestamp = CommonContext::host_timestamp(ctx)?;
        let host_height = CommonContext::host_height(ctx)?;

        let tm_consensus_state = TmConsensusState::try_from(consensus_state)?;

        ctx.store_client_state(ClientStatePath::new(client_id.clone()), self.clone().into())?;
        ctx.store_consensus_state(
            ClientConsensusStatePath::new(
                client_id.clone(),
                self.0.latest_height.revision_number(),
                self.0.latest_height.revision_height(),
            ),
            tm_consensus_state.into(),
        )?;
        ctx.store_update_meta(
            client_id.clone(),
            self.latest_height(),
            host_timestamp,
            host_height,
        )?;

        Ok(())
    }

    fn update_state(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        header: Any,
    ) -> Result<Vec<Height>, ClientError> {
        let header = TmHeader::try_from(header)?;
        let header_height = header.height();

        self.prune_oldest_consensus_state(ctx, client_id)?;

        let maybe_existing_consensus_state = {
            let path_at_header_height = ClientConsensusStatePath::new(
                client_id.clone(),
                header_height.revision_number(),
                header_height.revision_height(),
            );

            CommonContext::consensus_state(ctx, &path_at_header_height).ok()
        };

        if maybe_existing_consensus_state.is_some() {
            // if we already had the header installed by a previous relayer
            // then this is a no-op.
            //
            // Do nothing.
        } else {
            let host_timestamp = CommonContext::host_timestamp(ctx)?;
            let host_height = CommonContext::host_height(ctx)?;

            let new_consensus_state = ConsensusStateType::from(header.clone());
            let new_client_state = self.0.clone().with_header(header)?;

            ctx.store_consensus_state(
                ClientConsensusStatePath::new(
                    client_id.clone(),
                    header_height.revision_number(),
                    header_height.revision_height(),
                ),
                TmConsensusState::from(new_consensus_state).into(),
            )?;
            ctx.store_client_state(
                ClientStatePath::new(client_id.clone()),
                ClientState::from(new_client_state).into(),
            )?;
            ctx.store_update_meta(
                client_id.clone(),
                header_height,
                host_timestamp,
                host_height,
            )?;
        }

        Ok(vec![header_height])
    }

    fn update_state_on_misbehaviour(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        _client_message: Any,
    ) -> Result<(), ClientError> {
        // NOTE: frozen height is  set to `Height {revision_height: 0,
        // revision_number: 1}` and it is the same for all misbehaviour. This
        // aligns with the
        // [`ibc-go`](https://github.com/cosmos/ibc-go/blob/0e3f428e66d6fc0fc6b10d2f3c658aaa5000daf7/modules/light-clients/07-tendermint/misbehaviour.go#L18-L19)
        // implementation.
        let frozen_client_state = self.0.clone().with_frozen_height(Height::min(0));

        let wrapped_frozen_client_state = ClientState::from(frozen_client_state);

        ctx.store_client_state(
            ClientStatePath::new(client_id.clone()),
            wrapped_frozen_client_state.into(),
        )?;

        Ok(())
    }

    // Commit the new client state and consensus state to the store
    fn update_state_on_upgrade(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
    ) -> Result<Height, ClientError> {
        let mut upgraded_tm_client_state = Self::try_from(upgraded_client_state)?;
        let upgraded_tm_cons_state = TmConsensusState::try_from(upgraded_consensus_state)?;

        upgraded_tm_client_state.0.zero_custom_fields();

        // Construct new client state and consensus state relayer chosen client
        // parameters are ignored. All chain-chosen parameters come from
        // committed client, all client-chosen parameters come from current
        // client.
        let new_client_state = ClientStateType::new(
            upgraded_tm_client_state.0.chain_id,
            self.0.trust_level,
            self.0.trusting_period,
            upgraded_tm_client_state.0.unbonding_period,
            self.0.max_clock_drift,
            upgraded_tm_client_state.0.latest_height,
            upgraded_tm_client_state.0.proof_specs,
            upgraded_tm_client_state.0.upgrade_path,
            self.0.allow_update,
        )?;

        // The new consensus state is merely used as a trusted kernel against
        // which headers on the new chain can be verified. The root is just a
        // stand-in sentinel value as it cannot be known in advance, thus no
        // proof verification will pass. The timestamp and the
        // NextValidatorsHash of the consensus state is the blocktime and
        // NextValidatorsHash of the last block committed by the old chain. This
        // will allow the first block of the new chain to be verified against
        // the last validators of the old chain so long as it is submitted
        // within the TrustingPeriod of this client.
        // NOTE: We do not set processed time for this consensus state since
        // this consensus state should not be used for packet verification as
        // the root is empty. The next consensus state submitted using update
        // will be usable for packet-verification.
        let sentinel_root = "sentinel_root".as_bytes().to_vec();
        let new_consensus_state = ConsensusStateType::new(
            sentinel_root.into(),
            upgraded_tm_cons_state.timestamp(),
            upgraded_tm_cons_state.next_validators_hash(),
        );

        let latest_height = new_client_state.latest_height;
        let host_timestamp = CommonContext::host_timestamp(ctx)?;
        let host_height = CommonContext::host_height(ctx)?;

        ctx.store_client_state(
            ClientStatePath::new(client_id.clone()),
            ClientState::from(new_client_state).into(),
        )?;
        ctx.store_consensus_state(
            ClientConsensusStatePath::new(
                client_id.clone(),
                latest_height.revision_number(),
                latest_height.revision_height(),
            ),
            TmConsensusState::from(new_consensus_state).into(),
        )?;
        ctx.store_update_meta(
            client_id.clone(),
            latest_height,
            host_timestamp,
            host_height,
        )?;

        Ok(latest_height)
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use ibc_client_tendermint_types::{
        AllowUpdate, ClientState as ClientStateType, TrustThreshold,
    };
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
            unbonding_period: Duration::new(128000, 0),
            max_clock_drift: Duration::new(3, 0),
            latest_height: Height::new(1, 10).expect("Never fails"),
            proof_specs: ProofSpecs::default(),
            upgrade_path: Default::default(),
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
            let res = client_state.validate_proof_height(test.height);

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
