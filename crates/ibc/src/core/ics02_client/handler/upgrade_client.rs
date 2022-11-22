//! Protocol logic specific to processing ICS2 messages of type `MsgUpgradeAnyClient`.
//!
use crate::core::ics02_client::client_state::{ClientState, UpdatedState};
use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::core::ics02_client::context::ClientReader;
use crate::core::ics02_client::error::Error;
use crate::core::ics02_client::events::UpgradeClient;
use crate::core::ics02_client::handler::ClientResult;
use crate::core::ics02_client::msgs::upgrade_client::MsgUpgradeClient;
use crate::core::ics24_host::identifier::ClientId;
use crate::core::ics24_host::path::{ClientConsensusStatePath, ClientStatePath};
use crate::core::{ExecutionContext, ValidationContext};
use crate::events::IbcEvent;
use crate::handler::{HandlerOutput, HandlerResult};
use crate::prelude::*;

/// The result following the successful processing of a `MsgUpgradeAnyClient` message.
#[derive(Clone, Debug, PartialEq)]
pub struct UpgradeClientResult {
    pub client_id: ClientId,
    pub client_state: Box<dyn ClientState>,
    pub consensus_state: Box<dyn ConsensusState>,
}

pub(crate) fn validate<Ctx>(ctx: &Ctx, msg: MsgUpgradeClient) -> Result<(), Error>
where
    Ctx: ValidationContext,
{
    let MsgUpgradeClient { client_id, .. } = msg;

    // Read client state from the host chain store.
    let old_client_state = ctx.client_state(&client_id)?;

    if old_client_state.is_frozen() {
        return Err(Error::client_frozen(client_id));
    }

    let upgrade_client_state = ctx.decode_client_state(msg.client_state)?;

    if old_client_state.latest_height() >= upgrade_client_state.latest_height() {
        return Err(Error::low_upgrade_height(
            old_client_state.latest_height(),
            upgrade_client_state.latest_height(),
        ));
    }

    Ok(())
}

pub(crate) fn execute<Ctx>(ctx: &mut Ctx, msg: MsgUpgradeClient) -> Result<(), Error>
where
    Ctx: ExecutionContext,
{
    let MsgUpgradeClient { client_id, .. } = msg;

    let upgrade_client_state = ctx.decode_client_state(msg.client_state)?;

    let UpdatedState {
        client_state,
        consensus_state,
    } = upgrade_client_state.verify_upgrade_and_update_state(
        msg.consensus_state.clone(),
        msg.proof_upgrade_client.clone(),
        msg.proof_upgrade_consensus_state,
    )?;

    // Not implemented yet: https://github.com/informalsystems/ibc-rs/issues/722
    // todo!()

    ctx.store_client_state(ClientStatePath(client_id.clone()), client_state.clone())?;
    ctx.store_consensus_state(
        ClientConsensusStatePath::new(client_id.clone(), client_state.latest_height()),
        consensus_state,
    )?;

    ctx.emit_ibc_event(IbcEvent::UpgradeClient(UpgradeClient::new(
        client_id,
        client_state.client_type(),
        client_state.latest_height(),
    )));

    Ok(())
}

pub fn process(
    ctx: &dyn ClientReader,
    msg: MsgUpgradeClient,
) -> HandlerResult<ClientResult, Error> {
    let mut output = HandlerOutput::builder();
    let MsgUpgradeClient { client_id, .. } = msg;

    // Read client state from the host chain store.
    let old_client_state = ctx.client_state(&client_id)?;

    if old_client_state.is_frozen() {
        return Err(Error::client_frozen(client_id));
    }

    let upgrade_client_state = ctx.decode_client_state(msg.client_state)?;

    if old_client_state.latest_height() >= upgrade_client_state.latest_height() {
        return Err(Error::low_upgrade_height(
            old_client_state.latest_height(),
            upgrade_client_state.latest_height(),
        ));
    }

    let UpdatedState {
        client_state,
        consensus_state,
    } = upgrade_client_state.verify_upgrade_and_update_state(
        msg.consensus_state.clone(),
        msg.proof_upgrade_client.clone(),
        msg.proof_upgrade_consensus_state,
    )?;

    // Not implemented yet: https://github.com/informalsystems/ibc-rs/issues/722
    // todo!()

    let client_type = client_state.client_type();
    let consensus_height = client_state.latest_height();

    let result = ClientResult::Upgrade(UpgradeClientResult {
        client_id: client_id.clone(),
        client_state,
        consensus_state,
    });

    output.emit(IbcEvent::UpgradeClient(UpgradeClient::new(
        client_id,
        client_type,
        consensus_height,
    )));

    Ok(output.with_result(result))
}

#[cfg(test)]
mod tests {
    use crate::events::IbcEvent;
    use crate::{downcast, prelude::*};

    use core::str::FromStr;

    use crate::core::ics02_client::error::{Error, ErrorDetail};
    use crate::core::ics02_client::handler::dispatch;
    use crate::core::ics02_client::handler::ClientResult::Upgrade;
    use crate::core::ics02_client::msgs::upgrade_client::MsgUpgradeClient;
    use crate::core::ics02_client::msgs::ClientMsg;
    use crate::core::ics24_host::identifier::ClientId;
    use crate::handler::HandlerOutput;
    use crate::mock::client_state::client_type as mock_client_type;
    use crate::mock::client_state::MockClientState;
    use crate::mock::consensus_state::MockConsensusState;
    use crate::mock::context::MockContext;
    use crate::mock::header::MockHeader;
    use crate::test_utils::get_dummy_account_id;
    use crate::Height;

    #[test]
    fn test_upgrade_client_ok() {
        let client_id = ClientId::default();
        let signer = get_dummy_account_id();

        let ctx = MockContext::default().with_client(&client_id, Height::new(0, 42).unwrap());

        let msg = MsgUpgradeClient {
            client_id: client_id.clone(),
            client_state: MockClientState::new(MockHeader::new(Height::new(1, 26).unwrap())).into(),
            consensus_state: MockConsensusState::new(MockHeader::new(Height::new(1, 26).unwrap()))
                .into(),
            proof_upgrade_client: Default::default(),
            proof_upgrade_consensus_state: Default::default(),
            signer,
        };

        let output = dispatch(&ctx, ClientMsg::UpgradeClient(msg.clone()));

        match output {
            Ok(HandlerOutput {
                result,
                events: _,
                log,
            }) => {
                assert!(log.is_empty());
                // Check the result
                match result {
                    Upgrade(upg_res) => {
                        assert_eq!(upg_res.client_id, client_id);
                        assert_eq!(upg_res.client_state.as_ref().clone_into(), msg.client_state)
                    }
                    _ => panic!("upgrade handler result has incorrect type"),
                }
            }
            Err(err) => {
                panic!("unexpected error: {}", err);
            }
        }
    }

    #[test]
    fn test_upgrade_nonexisting_client() {
        let client_id = ClientId::from_str("mockclient1").unwrap();
        let signer = get_dummy_account_id();

        let ctx = MockContext::default().with_client(&client_id, Height::new(0, 42).unwrap());

        let msg = MsgUpgradeClient {
            client_id: ClientId::from_str("nonexistingclient").unwrap(),
            client_state: MockClientState::new(MockHeader::new(Height::new(1, 26).unwrap())).into(),
            consensus_state: MockConsensusState::new(MockHeader::new(Height::new(1, 26).unwrap()))
                .into(),
            proof_upgrade_client: Default::default(),
            proof_upgrade_consensus_state: Default::default(),
            signer,
        };

        let output = dispatch(&ctx, ClientMsg::UpgradeClient(msg.clone()));

        match output {
            Err(Error(ErrorDetail::ClientNotFound(e), _)) => {
                assert_eq!(e.client_id, msg.client_id);
            }
            _ => {
                panic!("expected ClientNotFound error, instead got {:?}", output);
            }
        }
    }

    #[test]
    fn test_upgrade_client_low_height() {
        let client_id = ClientId::default();
        let signer = get_dummy_account_id();

        let ctx = MockContext::default().with_client(&client_id, Height::new(0, 42).unwrap());

        let msg = MsgUpgradeClient {
            client_id,
            client_state: MockClientState::new(MockHeader::new(Height::new(0, 26).unwrap())).into(),
            consensus_state: MockConsensusState::new(MockHeader::new(Height::new(0, 26).unwrap()))
                .into(),
            proof_upgrade_client: Default::default(),
            proof_upgrade_consensus_state: Default::default(),
            signer,
        };

        let output = dispatch(&ctx, ClientMsg::UpgradeClient(msg.clone()));

        match output {
            Err(Error(ErrorDetail::LowUpgradeHeight(e), _)) => {
                assert_eq!(e.upgraded_height, Height::new(0, 42).unwrap());
                assert_eq!(
                    e.client_height,
                    MockClientState::try_from(msg.client_state)
                        .unwrap()
                        .latest_height()
                );
            }
            _ => {
                panic!("expected LowUpgradeHeight error, instead got {:?}", output);
            }
        }
    }

    #[test]
    fn test_upgrade_client_event() {
        let client_id = ClientId::default();
        let signer = get_dummy_account_id();

        let ctx = MockContext::default().with_client(&client_id, Height::new(0, 42).unwrap());

        let upgrade_height = Height::new(1, 26).unwrap();
        let msg = MsgUpgradeClient {
            client_id: client_id.clone(),
            client_state: MockClientState::new(MockHeader::new(upgrade_height)).into(),
            consensus_state: MockConsensusState::new(MockHeader::new(upgrade_height)).into(),
            proof_upgrade_client: Default::default(),
            proof_upgrade_consensus_state: Default::default(),
            signer,
        };

        let output = dispatch(&ctx, ClientMsg::UpgradeClient(msg)).unwrap();
        let upgrade_client_event =
            downcast!(output.events.first().unwrap() => IbcEvent::UpgradeClient).unwrap();
        assert_eq!(upgrade_client_event.client_id(), &client_id);
        assert_eq!(upgrade_client_event.client_type(), &mock_client_type());
        assert_eq!(upgrade_client_event.consensus_height(), &upgrade_height);
    }
}
