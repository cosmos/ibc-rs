//! Protocol logic specific to processing ICS2 messages of type `MsgUpgradeAnyClient`.
//!
use crate::prelude::*;

use crate::core::ics02_client::client_state::{ClientState, UpdatedState};
use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::core::ics02_client::context::ClientReader;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics02_client::events::UpgradeClient;
use crate::core::ics02_client::handler::ClientResult;
use crate::core::ics02_client::msgs::upgrade_client::MsgUpgradeClient;
use crate::core::ics24_host::identifier::ClientId;
use crate::events::IbcEvent;
use crate::handler::{HandlerOutput, HandlerResult};

#[cfg(feature = "val_exec_ctx")]
use crate::core::context::ContextError;
#[cfg(feature = "val_exec_ctx")]
use crate::core::ics24_host::path::{ClientConsensusStatePath, ClientStatePath};
#[cfg(feature = "val_exec_ctx")]
use crate::core::{ExecutionContext, ValidationContext};

/// The result following the successful processing of a `MsgUpgradeAnyClient` message.
#[derive(Clone, Debug, PartialEq)]
pub struct UpgradeClientResult {
    pub client_id: ClientId,
    pub client_state: Box<dyn ClientState>,
    pub consensus_state: Box<dyn ConsensusState>,
}

#[cfg(feature = "val_exec_ctx")]
pub(crate) fn validate<Ctx>(ctx: &Ctx, msg: MsgUpgradeClient) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let MsgUpgradeClient { client_id, .. } = msg;

    // Temporary has been disabled until we have a better understanding of some design implications
    if cfg!(feature = "disable_upgrade_client") {
        return Err(ContextError::ClientError(ClientError::Other {
            description: "upgrade_client feature is not supported".to_string(),
        }));
    }

    // Read the current latest client state from the host chain store.
    let old_client_state = ctx.client_state(&client_id)?;

    // Check if the client is frozen.
    if old_client_state.is_frozen() {
        return Err(ContextError::ClientError(ClientError::ClientFrozen {
            client_id,
        }));
    }

    // Read the latest consensus state from the host chain store.
    let old_consensus_state = ctx
        .consensus_state(&client_id, &old_client_state.latest_height())
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

#[cfg(feature = "val_exec_ctx")]
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

pub(crate) fn process(
    ctx: &dyn ClientReader,
    msg: MsgUpgradeClient,
) -> HandlerResult<ClientResult, ClientError> {
    let mut output = HandlerOutput::builder();
    let MsgUpgradeClient { client_id, .. } = msg;

    // Temporary has been disabled until we have a better understanding of some design implications
    if !cfg!(feature = "upgrade_client") {
        return Err(ClientError::Other {
            description: "upgrade_client feature is not supported".to_string(),
        });
    }

    // Read the current latest client state from the host chain store.
    let old_client_state = ctx.client_state(&client_id)?;

    // Check if the client is frozen.
    if old_client_state.is_frozen() {
        return Err(ClientError::ClientFrozen { client_id });
    }

    // Read the latest consensus state from the host chain store.
    let old_consensus_state = ctx
        .consensus_state(&client_id, &old_client_state.latest_height())
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
        return Err(ClientError::HeaderNotWithinTrustPeriod {
            latest_time: old_consensus_state.timestamp(),
            update_time: now,
        });
    };

    // Validate the upgraded client state and consensus state and verify proofs against the root
    old_client_state.verify_upgrade_client(
        msg.client_state.clone(),
        msg.consensus_state.clone(),
        msg.proof_upgrade_client.clone(),
        msg.proof_upgrade_consensus_state.clone(),
        old_consensus_state.root(),
    )?;

    // Create updated new client state and consensus state
    let UpdatedState {
        client_state,
        consensus_state,
    } = old_client_state
        .update_state_with_upgrade_client(msg.client_state.clone(), msg.consensus_state)?;

    let result = ClientResult::Upgrade(UpgradeClientResult {
        client_id: client_id.clone(),
        client_state: client_state.clone(),
        consensus_state,
    });

    output.emit(IbcEvent::UpgradeClient(UpgradeClient::new(
        client_id,
        client_state.client_type(),
        client_state.latest_height(),
    )));

    Ok(output.with_result(result))
}

#[cfg(feature = "upgrade_client")]
#[cfg(test)]
mod tests {
    use crate::events::IbcEvent;
    use crate::{downcast, prelude::*};

    use core::str::FromStr;

    use crate::core::ics02_client::error::ClientError;
    use crate::core::ics02_client::handler::dispatch;
    use crate::core::ics02_client::handler::ClientResult::Upgrade;
    use crate::core::ics02_client::msgs::upgrade_client::MsgUpgradeClient;
    use crate::core::ics02_client::msgs::ClientMsg;
    use crate::core::ics24_host::identifier::ClientId;
    use crate::mock::client_state::client_type as mock_client_type;
    use crate::mock::client_state::MockClientState;
    use crate::mock::consensus_state::MockConsensusState;
    use crate::mock::context::MockContext;
    use crate::mock::header::MockHeader;
    use crate::test_utils::get_dummy_account_id;
    use crate::Height;

    #[test]
    fn upgrade_client_msg_processing() {
        struct Test {
            name: String,
            ctx: MockContext,
            msg: MsgUpgradeClient,
            want_pass: bool,
        }
        let client_id = ClientId::default();
        let signer = get_dummy_account_id();
        let ctx = MockContext::default().with_client(&client_id, Height::new(0, 42).unwrap());
        let tests: Vec<Test> = vec![
            Test {
                name: "Processing succeeds".to_string(),
                ctx: ctx.clone(),
                msg: MsgUpgradeClient {
                    client_id: client_id.clone(),
                    client_state: MockClientState::new(MockHeader::new(
                        Height::new(1, 26).unwrap(),
                    ))
                    .into(),
                    consensus_state: MockConsensusState::new(MockHeader::new(
                        Height::new(1, 26).unwrap(),
                    ))
                    .into(),
                    proof_upgrade_client: Default::default(),
                    proof_upgrade_consensus_state: Default::default(),
                    signer: signer.clone(),
                },
                want_pass: true,
            },
            Test {
                name: "Processing fails for non existing client".to_string(),
                ctx: ctx.clone(),
                msg: MsgUpgradeClient {
                    client_id: ClientId::from_str("nonexistingclient").unwrap(),
                    client_state: MockClientState::new(MockHeader::new(
                        Height::new(1, 26).unwrap(),
                    ))
                    .into(),
                    consensus_state: MockConsensusState::new(MockHeader::new(
                        Height::new(1, 26).unwrap(),
                    ))
                    .into(),
                    proof_upgrade_client: Default::default(),
                    proof_upgrade_consensus_state: Default::default(),
                    signer: signer.clone(),
                },
                want_pass: false,
            },
            Test {
                name: "Processing fails for low upgrade height".to_string(),
                ctx,
                msg: MsgUpgradeClient {
                    client_id: client_id.clone(),
                    client_state: MockClientState::new(MockHeader::new(
                        Height::new(0, 26).unwrap(),
                    ))
                    .into(),
                    consensus_state: MockConsensusState::new(MockHeader::new(
                        Height::new(0, 26).unwrap(),
                    ))
                    .into(),
                    proof_upgrade_client: Default::default(),
                    proof_upgrade_consensus_state: Default::default(),
                    signer,
                },
                want_pass: false,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let output = dispatch(&test.ctx, ClientMsg::UpgradeClient(test.msg.clone()));
            let test_name = test.name;
            match (test.want_pass, output) {
                (true, Ok(output)) => {
                    let upgrade_client_event =
                        downcast!(output.events.first().unwrap() => IbcEvent::UpgradeClient)
                            .unwrap();
                    assert_eq!(upgrade_client_event.client_id(), &client_id);
                    assert_eq!(upgrade_client_event.client_type(), &mock_client_type());
                    assert_eq!(
                        upgrade_client_event.consensus_height(),
                        &Height::new(1, 26).unwrap()
                    );
                    assert!(output.log.is_empty());
                    match output.result {
                        Upgrade(upg_res) => {
                            assert_eq!(upg_res.client_id, client_id);
                            assert_eq!(
                                upg_res.client_state.as_ref().clone_into(),
                                test.msg.client_state
                            );
                            assert_eq!(
                                upg_res.consensus_state.as_ref().clone_into(),
                                test.msg.consensus_state
                            );
                        }
                        _ => panic!("upgrade handler result has incorrect type"),
                    }
                }
                (true, Err(e)) => panic!("unexpected error for test {test_name}, {e:?}"),
                (false, Err(e)) => match e {
                    ClientError::ClientNotFound { client_id } => {
                        assert_eq!(client_id, test.msg.client_id)
                    }
                    ClientError::LowUpgradeHeight {
                        upgraded_height,
                        client_height,
                    } => {
                        assert_eq!(upgraded_height, Height::new(0, 42).unwrap());
                        assert_eq!(
                            client_height,
                            MockClientState::try_from(test.msg.client_state)
                                .unwrap()
                                .latest_height()
                        );
                    }
                    _ => panic!("unexpected error for test {test_name}, {e:?}"),
                },
                (false, Ok(e)) => {
                    panic!("unexpected success for test {test_name}, result: {e:?}")
                }
            }
        }
    }
}
