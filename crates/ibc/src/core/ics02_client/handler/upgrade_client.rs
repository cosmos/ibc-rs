//! Protocol logic specific to processing ICS2 messages of type `MsgUpgradeAnyClient`.
//!
use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::prelude::*;

use crate::core::context::ContextError;
use crate::core::events::{IbcEvent, MessageEvent};
use crate::core::ics02_client::client_state::{ClientStateCommon, ClientStateExecution};
use crate::core::ics02_client::error::ClientError;
use crate::core::ics02_client::events::UpgradeClient;
use crate::core::ics02_client::msgs::upgrade_client::MsgUpgradeClient;
use crate::core::ics23_commitment::merkle::MerkleProof;
use crate::core::ics24_host::path::ClientConsensusStatePath;
use crate::core::{ExecutionContext, ValidationContext};

pub(crate) fn validate<Ctx>(ctx: &Ctx, msg: MsgUpgradeClient) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let MsgUpgradeClient {
        client_id, signer, ..
    } = msg;

    ctx.validate_message_signer(&signer)?;

    // Read the current latest client state from the host chain store.
    let old_client_state = ctx.client_state(&client_id)?;

    // Check if the client is frozen.
    old_client_state.confirm_not_frozen()?;

    // Read the latest consensus state from the host chain store.
    let old_client_cons_state_path =
        ClientConsensusStatePath::new(&client_id, &old_client_state.latest_height());
    let old_consensus_state = ctx
        .consensus_state(&old_client_cons_state_path)
        .map_err(|_| ClientError::ConsensusStateNotFound {
            client_id: client_id.clone(),
            height: old_client_state.latest_height(),
        })?;

    let now = ctx.host_timestamp()?;
    let duration = now
        .duration_since(&old_consensus_state.timestamp())
        .ok_or_else(|| ClientError::InvalidConsensusStateTimestamp {
            time1: old_consensus_state.timestamp(),
            time2: now,
        })?;

    // Check if the latest consensus state is within the trust period.
    if old_client_state.expired(duration) {
        return Err(ContextError::ClientError(
            ClientError::HeaderNotWithinTrustPeriod {
                latest_time: old_consensus_state.timestamp(),
                update_time: now,
            },
        ));
    };

    // Note: verification of proofs that unmarshalled correctly has been done
    // while decoding the proto message into a `MsgEnvelope` domain type
    let merkle_proof_upgrade_client = MerkleProof::from(msg.proof_upgrade_client.clone());
    let merkle_proof_upgrade_cons_state = MerkleProof::from(msg.proof_upgrade_consensus_state);

    // Validate the upgraded client state and consensus state and verify proofs against the root
    old_client_state.verify_upgrade_client(
        msg.client_state.clone(),
        msg.consensus_state,
        merkle_proof_upgrade_client,
        merkle_proof_upgrade_cons_state,
        old_consensus_state.root(),
    )?;

    Ok(())
}

pub(crate) fn execute<Ctx>(ctx: &mut Ctx, msg: MsgUpgradeClient) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    let MsgUpgradeClient { client_id, .. } = msg;

    let old_client_state = ctx.client_state(&client_id)?;

    let latest_height = old_client_state.update_state_with_upgrade_client(
        ctx.get_client_execution_context(),
        &client_id,
        msg.client_state.clone(),
        msg.consensus_state,
    )?;

    let event = IbcEvent::UpgradeClient(UpgradeClient::new(
        client_id,
        old_client_state.client_type(),
        latest_height,
    ));
    ctx.emit_ibc_event(IbcEvent::Message(MessageEvent::Client));
    ctx.emit_ibc_event(event);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::clients::ics07_tendermint::client_state::ClientState as TmClientState;
    use crate::clients::ics07_tendermint::client_type;
    use crate::clients::ics07_tendermint::header::test_util::get_dummy_tendermint_header;

    use crate::core::ics02_client::error::UpgradeClientError;
    use crate::core::ics02_client::ClientValidationContext;
    use crate::core::ics03_connection::handler::test_util::{Expect, Fixture};
    use crate::core::ics24_host::identifier::ClientId;
    use crate::downcast;
    use crate::Height;

    use crate::mock::client_state::client_type as mock_client_type;
    use crate::mock::context::{AnyClientState, AnyConsensusState, MockContext};

    enum Ctx {
        Default,
        WithClient,
    }

    enum Msg {
        Default,
        LowUpgradeHeight,
        UnknownUpgradedClientStateType,
    }

    fn msg_upgrade_client_fixture(ctx_variant: Ctx, msg_variant: Msg) -> Fixture<MsgUpgradeClient> {
        let client_id = ClientId::new(mock_client_type(), 0).unwrap();

        let ctx_default = MockContext::default();
        let ctx_with_client = ctx_default
            .clone()
            .with_client(&client_id, Height::new(0, 42).unwrap());
        let ctx = match ctx_variant {
            Ctx::Default => ctx_default,
            Ctx::WithClient => ctx_with_client,
        };

        let upgrade_height = Height::new(1, 26).unwrap();
        let msg_default = MsgUpgradeClient::new_dummy(upgrade_height);

        let low_upgrade_height = Height::new(0, 26).unwrap();
        let msg_with_low_upgrade_height = MsgUpgradeClient::new_dummy(low_upgrade_height);

        let msg_with_unknown_upgraded_cs = MsgUpgradeClient {
            client_state: TmClientState::new_dummy_from_header(get_dummy_tendermint_header())
                .into(),
            ..msg_default.clone()
        };

        let msg = match msg_variant {
            Msg::Default => msg_default,
            Msg::LowUpgradeHeight => msg_with_low_upgrade_height,
            Msg::UnknownUpgradedClientStateType => msg_with_unknown_upgraded_cs,
        };

        Fixture { ctx, msg }
    }

    fn upgrade_client_validate(fxt: &Fixture<MsgUpgradeClient>, expect: Expect) {
        let Fixture { ctx, msg } = fxt;
        let res = validate(ctx, msg.clone());
        let err_msg = fxt.generate_error_msg(&expect, "validation", &res);

        match expect {
            Expect::Failure(_) => {
                assert!(res.is_err(), "{err_msg}");
            }
            Expect::Success => {
                assert!(res.is_ok(), "{err_msg}");
            }
        };
    }

    fn upgrade_client_execute(fxt: &mut Fixture<MsgUpgradeClient>, expect: Expect) {
        let res = execute(&mut fxt.ctx, fxt.msg.clone());
        let err_msg = fxt.generate_error_msg(&expect, "execution", &res);
        match expect {
            Expect::Failure(err) => {
                assert!(res.is_err(), "{err_msg}");
                assert_eq!(
                    core::mem::discriminant(res.as_ref().unwrap_err()),
                    core::mem::discriminant(&err.unwrap())
                );
            }
            Expect::Success => {
                assert!(res.is_ok(), "{err_msg}");
                assert!(matches!(
                    fxt.ctx.events[0],
                    IbcEvent::Message(MessageEvent::Client)
                ));
                let upgrade_client_event =
                    downcast!(&fxt.ctx.events[1] => IbcEvent::UpgradeClient).unwrap();
                let plan_height = Height::new(1, 26).unwrap();

                assert_eq!(upgrade_client_event.client_id(), &fxt.msg.client_id);
                assert_eq!(upgrade_client_event.client_type(), &mock_client_type());
                assert_eq!(upgrade_client_event.consensus_height(), &plan_height);

                let client_state = fxt.ctx.client_state(&fxt.msg.client_id).unwrap();
                let msg_client_state: AnyClientState =
                    fxt.msg.client_state.clone().try_into().unwrap();
                assert_eq!(client_state, msg_client_state);

                let consensus_state = fxt
                    .ctx
                    .consensus_state(&ClientConsensusStatePath::new(
                        &fxt.msg.client_id,
                        &plan_height,
                    ))
                    .unwrap();
                let msg_consensus_state: AnyConsensusState =
                    fxt.msg.consensus_state.clone().try_into().unwrap();
                assert_eq!(consensus_state, msg_consensus_state);
            }
        };
    }

    #[test]
    fn msg_upgrade_client_healthy() {
        let mut fxt = msg_upgrade_client_fixture(Ctx::WithClient, Msg::Default);
        upgrade_client_validate(&fxt, Expect::Success);
        upgrade_client_execute(&mut fxt, Expect::Success);
    }

    #[test]
    fn upgrade_client_fail_nonexisting_client() {
        let fxt = msg_upgrade_client_fixture(Ctx::Default, Msg::Default);
        let expected_err = ContextError::ClientError(ClientError::ClientStateNotFound {
            client_id: fxt.msg.client_id.clone(),
        });
        upgrade_client_validate(&fxt, Expect::Failure(Some(expected_err)));
    }

    #[test]
    fn upgrade_client_fail_low_upgrade_height() {
        let fxt: Fixture<MsgUpgradeClient> =
            msg_upgrade_client_fixture(Ctx::WithClient, Msg::LowUpgradeHeight);
        let expected_err: ClientError = UpgradeClientError::LowUpgradeHeight {
            upgraded_height: Height::new(0, 26).unwrap(),
            client_height: fxt.ctx.latest_height(),
        }
        .into();
        upgrade_client_validate(&fxt, Expect::Failure(Some(expected_err.into())));
    }

    #[test]
    fn upgrade_client_fail_unknown_upgraded_client_state() {
        let fxt = msg_upgrade_client_fixture(Ctx::WithClient, Msg::UnknownUpgradedClientStateType);
        let expected_err = ContextError::ClientError(ClientError::UnknownClientStateType {
            client_state_type: client_type().to_string(),
        });
        upgrade_client_validate(&fxt, Expect::Failure(Some(expected_err)));
    }
}
