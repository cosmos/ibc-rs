//! Protocol logic specific to processing ICS2 messages of type `MsgUpgradeAnyClient`.
//!
use crate::prelude::*;

use crate::core::ics02_client::client_state::UpdatedState;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics02_client::events::UpgradeClient;
use crate::core::ics02_client::msgs::upgrade_client::MsgUpgradeClient;
use crate::events::IbcEvent;

use crate::core::context::ContextError;

use crate::core::ics24_host::path::{ClientConsensusStatePath, ClientStatePath};

use crate::core::{ExecutionContext, ValidationContext};

pub(crate) fn validate<Ctx>(ctx: &Ctx, msg: MsgUpgradeClient) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let MsgUpgradeClient { client_id, .. } = msg;

    // Temporary has been disabled until we have a better understanding of some design implications
    if cfg!(not(feature = "upgrade_client")) {
        return Err(ContextError::ClientError(ClientError::Other {
            description: "upgrade_client feature is not supported".to_string(),
        }));
    }

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

    // Validate the upgraded client state and consensus state and verify proofs against the root
    old_client_state.verify_upgrade_client(
        msg.client_state.clone(),
        msg.consensus_state.clone(),
        msg.proof_upgrade_client.clone(),
        msg.proof_upgrade_consensus_state,
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

    let UpdatedState {
        client_state,
        consensus_state,
    } = old_client_state
        .update_state_with_upgrade_client(msg.client_state.clone(), msg.consensus_state)?;

    ctx.store_client_state(ClientStatePath::new(&client_id), client_state.clone())?;
    ctx.store_consensus_state(
        ClientConsensusStatePath::new(&client_id, &client_state.latest_height()),
        consensus_state,
    )?;

    let event = IbcEvent::UpgradeClient(UpgradeClient::new(
        client_id,
        client_state.client_type(),
        client_state.latest_height(),
    ));
    ctx.emit_ibc_event(IbcEvent::Message(event.event_type()));
    ctx.emit_ibc_event(event);

    Ok(())
}

#[cfg(feature = "upgrade_client")]
#[cfg(test)]
mod tests {
    use crate::core::ics02_client::handler::upgrade_client::execute;
    use crate::core::ics24_host::path::ClientConsensusStatePath;
    use crate::core::ValidationContext;
    use crate::events::{IbcEvent, IbcEventType};
    use crate::{downcast, prelude::*};
    use rstest::*;

    use core::str::FromStr;

    use crate::core::ics02_client::msgs::upgrade_client::MsgUpgradeClient;
    use crate::core::ics24_host::identifier::ClientId;
    use crate::mock::client_state::client_type as mock_client_type;
    use crate::mock::client_state::MockClientState;
    use crate::mock::consensus_state::MockConsensusState;
    use crate::mock::context::MockContext;
    use crate::mock::header::MockHeader;
    use crate::test_utils::get_dummy_account_id;
    use crate::Height;

    use super::validate;

    pub struct Fixture {
        pub ctx: MockContext,
        pub msg: MsgUpgradeClient,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let ctx =
            MockContext::default().with_client(&ClientId::default(), Height::new(0, 42).unwrap());

        let msg = MsgUpgradeClient {
            client_id: ClientId::default(),
            client_state: MockClientState::new(MockHeader::new(Height::new(1, 26).unwrap())).into(),
            consensus_state: MockConsensusState::new(MockHeader::new(Height::new(1, 26).unwrap()))
                .into(),
            proof_upgrade_client: Default::default(),
            proof_upgrade_consensus_state: Default::default(),
            signer: get_dummy_account_id(),
        };

        Fixture { ctx, msg }
    }

    #[rstest]
    fn upgrade_client_ok(fixture: Fixture) {
        let Fixture { mut ctx, msg } = fixture;

        let res = validate(&ctx, msg.clone());
        assert!(res.is_ok(), "validation happy path");

        let res = execute(&mut ctx, msg.clone());
        assert!(res.is_ok(), "execution happy path");

        assert!(matches!(
            ctx.events[0],
            IbcEvent::Message(IbcEventType::UpgradeClient)
        ));
        let upgrade_client_event = downcast!(&ctx.events[1] => IbcEvent::UpgradeClient).unwrap();
        assert_eq!(upgrade_client_event.client_id(), &msg.client_id);
        assert_eq!(upgrade_client_event.client_type(), &mock_client_type());
        assert_eq!(
            upgrade_client_event.consensus_height(),
            &Height::new(1, 26).unwrap()
        );

        let client_state = ctx.client_state(&msg.client_id).unwrap();
        assert_eq!(client_state.as_ref().clone_into(), msg.client_state);

        let consensus_state = ctx
            .consensus_state(&ClientConsensusStatePath {
                client_id: msg.client_id,
                epoch: 1,
                height: 26,
            })
            .unwrap();
        assert_eq!(consensus_state.as_ref().clone_into(), msg.consensus_state);
    }

    #[rstest]
    fn upgrade_client_fail_nonexisting_client(fixture: Fixture) {
        let Fixture { ctx, mut msg } = fixture;

        msg.client_id = ClientId::from_str("nonexistingclient").unwrap();

        let res = validate(&ctx, msg);
        assert!(
            res.is_err(),
            "validation fails because the client is non-existing"
        );
    }

    #[rstest]
    fn upgrade_client_fail_low_upgrade_height(fixture: Fixture) {
        let Fixture { ctx, mut msg } = fixture;

        msg.client_state =
            MockClientState::new(MockHeader::new(Height::new(0, 26).unwrap())).into();
        msg.consensus_state =
            MockConsensusState::new(MockHeader::new(Height::new(0, 26).unwrap())).into();

        let res = validate(&ctx, msg);
        assert!(
            res.is_err(),
            "validation fails because the upgrade height is too low"
        );
    }
}
