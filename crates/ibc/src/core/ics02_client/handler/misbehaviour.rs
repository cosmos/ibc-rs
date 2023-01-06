//! Protocol logic specific to processing ICS2 messages of type `MsgSubmitMisbehaviour`.
//!
use crate::prelude::*;

use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics02_client::context::ClientReader;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics02_client::events::ClientMisbehaviour;
use crate::core::ics02_client::handler::ClientResult;
use crate::core::ics02_client::msgs::misbehaviour::MsgSubmitMisbehaviour;
use crate::core::ics24_host::identifier::ClientId;
use crate::events::IbcEvent;
use crate::handler::{HandlerOutput, HandlerResult};

#[cfg(val_exec_ctx)]
use crate::core::ics24_host::path::ClientStatePath;
#[cfg(val_exec_ctx)]
use crate::core::{ContextError, ExecutionContext, ValidationContext};

/// The result following the successful processing of a `MsgSubmitMisbehaviour` message.
#[derive(Clone, Debug, PartialEq)]
pub struct MisbehaviourResult {
    pub client_id: ClientId,
    pub client_state: Box<dyn ClientState>,
}

#[cfg(val_exec_ctx)]
pub(crate) fn validate<Ctx>(ctx: &Ctx, msg: MsgSubmitMisbehaviour) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let MsgSubmitMisbehaviour {
        client_id,
        misbehaviour,
        signer: _,
    } = msg;

    // Read client state from the host chain store.
    let client_state = ctx.client_state(&client_id)?;

    if client_state.is_frozen() {
        return Err(ClientError::ClientFrozen { client_id }.into());
    }

    let _ = client_state
        .new_check_misbehaviour_and_update_state(ctx, client_id.clone(), misbehaviour)
        .map_err(|e| ClientError::MisbehaviourHandlingFailure {
            reason: e.to_string(),
        })?;

    Ok(())
}

#[cfg(val_exec_ctx)]
pub(crate) fn execute<Ctx>(ctx: &mut Ctx, msg: MsgSubmitMisbehaviour) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    let MsgSubmitMisbehaviour {
        client_id,
        misbehaviour,
        signer: _,
    } = msg;

    // Read client state from the host chain store.
    let client_state = ctx.client_state(&client_id)?;

    if client_state.is_frozen() {
        return Err(ClientError::ClientFrozen { client_id }.into());
    }

    let client_state = client_state
        .new_check_misbehaviour_and_update_state(ctx, client_id.clone(), misbehaviour)
        .map_err(|e| ClientError::MisbehaviourHandlingFailure {
            reason: e.to_string(),
        })?;

    ctx.emit_ibc_event(IbcEvent::ClientMisbehaviour(ClientMisbehaviour::new(
        client_id.clone(),
        client_state.client_type(),
    )));

    ctx.store_client_state(ClientStatePath(client_id), client_state)
}

pub(crate) fn process(
    ctx: &dyn ClientReader,
    msg: MsgSubmitMisbehaviour,
) -> HandlerResult<ClientResult, ClientError> {
    let mut output = HandlerOutput::builder();

    let MsgSubmitMisbehaviour {
        client_id,
        misbehaviour,
        signer: _,
    } = msg;

    // Read client state from the host chain store.
    let client_state = ctx.client_state(&client_id)?;

    if client_state.is_frozen() {
        return Err(ClientError::ClientFrozen { client_id });
    }

    let client_state = client_state
        .check_misbehaviour_and_update_state(ctx, client_id.clone(), misbehaviour)
        .map_err(|e| ClientError::MisbehaviourHandlingFailure {
            reason: e.to_string(),
        })?;

    output.emit(IbcEvent::ClientMisbehaviour(ClientMisbehaviour::new(
        client_id.clone(),
        client_state.client_type(),
    )));

    let result = ClientResult::Misbehaviour(MisbehaviourResult {
        client_id,
        client_state,
    });

    Ok(output.with_result(result))
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;
    use test_log::test;

    use crate::clients::ics07_tendermint::client_type as tm_client_type;
    use crate::clients::ics07_tendermint::header::Header as TmHeader;
    use crate::clients::ics07_tendermint::misbehaviour::Misbehaviour as TmMisbehaviour;
    use crate::core::ics02_client::client_type::ClientType;
    use crate::core::ics02_client::error::ClientError;
    use crate::core::ics02_client::handler::dispatch;
    use crate::core::ics02_client::handler::ClientResult;
    use crate::core::ics02_client::msgs::misbehaviour::MsgSubmitMisbehaviour;
    use crate::core::ics02_client::msgs::ClientMsg;
    use crate::core::ics24_host::identifier::{ChainId, ClientId};
    use crate::events::IbcEvent;
    use crate::handler::HandlerOutput;
    use crate::mock::client_state::client_type as mock_client_type;
    use crate::mock::context::MockContext;
    use crate::mock::header::MockHeader;
    use crate::mock::host::{HostBlock, HostType};
    use crate::mock::misbehaviour::Misbehaviour as MockMisbehaviour;
    use crate::test_utils::get_dummy_account_id;
    use crate::timestamp::Timestamp;
    use crate::Height;
    use crate::{downcast, prelude::*};

    /// Panics unless the given handler output satisfies the following conditions -
    /// * The output result is of type client misbehaviour
    /// * The specified client is frozen and it's frozen height is set correctly
    /// * Only a single misbehaviour event was emitted with the specified client id and type
    /// * No logs were emitted
    fn ensure_misbehaviour_result(
        output: Result<HandlerOutput<ClientResult>, ClientError>,
        client_id: &ClientId,
        client_type: &ClientType,
    ) {
        match output {
            Ok(HandlerOutput {
                result: ClientResult::Misbehaviour(res),
                mut events,
                log,
            }) => {
                // check result
                assert_eq!(&res.client_id, client_id);
                assert!(res.client_state.is_frozen());
                assert_eq!(
                    res.client_state.frozen_height(),
                    Some(Height::new(0, 1).unwrap())
                );

                // check events
                let misbehaviour_client_event =
                    downcast!(events.pop().unwrap() => IbcEvent::ClientMisbehaviour).unwrap();
                assert!(events.is_empty());
                assert_eq!(misbehaviour_client_event.client_id(), client_id);
                assert_eq!(misbehaviour_client_event.client_type(), client_type);

                // check logs
                assert!(log.is_empty());
            }
            _ => panic!(
                "Expected client to be frozen due to misbehaviour, instead got result: {:?}",
                output
            ),
        }
    }

    /// Tests misbehaviour handling for the mock client.
    /// Misbehaviour evidence consists of identical headers - mock misbehaviour handler considers it
    /// a valid proof of misbehaviour
    #[test]
    fn test_misbehaviour_client_ok() {
        let client_id = ClientId::default();
        let timestamp = Timestamp::now();
        let height = Height::new(0, 46).unwrap();
        let msg = MsgSubmitMisbehaviour {
            client_id: client_id.clone(),
            misbehaviour: MockMisbehaviour {
                client_id: client_id.clone(),
                header1: MockHeader::new(height).with_timestamp(timestamp),
                header2: MockHeader::new(height).with_timestamp(timestamp),
            }
            .into(),
            signer: get_dummy_account_id(),
        };

        let ctx = MockContext::default().with_client(&client_id, Height::new(0, 42).unwrap());
        let output = dispatch(&ctx, ClientMsg::Misbehaviour(msg));
        ensure_misbehaviour_result(output, &client_id, &mock_client_type());
    }

    /// Tests misbehaviour handling failure for a non-existent client
    #[test]
    fn test_misbehaviour_nonexisting_client() {
        let client_id = ClientId::from_str("mockclient1").unwrap();
        let height = Height::new(0, 46).unwrap();
        let msg = MsgSubmitMisbehaviour {
            client_id: ClientId::from_str("nonexistingclient").unwrap(),
            misbehaviour: MockMisbehaviour {
                client_id: client_id.clone(),
                header1: MockHeader::new(height),
                header2: MockHeader::new(height),
            }
            .into(),
            signer: get_dummy_account_id(),
        };

        let ctx = MockContext::default().with_client(&client_id, Height::new(0, 42).unwrap());
        let output = dispatch(&ctx, ClientMsg::Misbehaviour(msg.clone()));
        match output {
            Err(ClientError::ClientNotFound { client_id }) => assert_eq!(client_id, msg.client_id),
            _ => panic!("expected ClientNotFound error, instead got {:?}", output),
        }
    }

    /// Tests misbehaviour handling for multiple clients
    #[test]
    fn test_misbehaviour_client_ok_multiple() {
        let client_ids = vec![
            ClientId::from_str("mockclient1").unwrap(),
            ClientId::from_str("mockclient2").unwrap(),
            ClientId::from_str("mockclient3").unwrap(),
        ];
        let signer = get_dummy_account_id();
        let initial_height = Height::new(0, 45).unwrap();
        let misbehaviour_height = Height::new(0, 49).unwrap();

        let mut ctx = MockContext::default();

        for cid in &client_ids {
            ctx = ctx.with_client(cid, initial_height);
        }

        for client_id in &client_ids {
            let msg = MsgSubmitMisbehaviour {
                client_id: client_id.clone(),
                misbehaviour: MockMisbehaviour {
                    client_id: client_id.clone(),
                    header1: MockHeader::new(misbehaviour_height),
                    header2: MockHeader::new(misbehaviour_height),
                }
                .into(),
                signer: signer.clone(),
            };

            let output = dispatch(&ctx, ClientMsg::Misbehaviour(msg.clone()));
            ensure_misbehaviour_result(output, client_id, &mock_client_type());
        }
    }

    /// Tests misbehaviour handling for the synthetic Tendermint client.
    /// Misbehaviour evidence consists of equivocal headers.
    #[test]
    fn test_misbehaviour_synthetic_tendermint_equivocation() {
        let client_id = ClientId::new(tm_client_type(), 0).unwrap();
        let client_height = Height::new(1, 20).unwrap();
        let misbehaviour_height = Height::new(1, 21).unwrap();
        let chain_id_b = ChainId::new("mockgaiaB".to_string(), 1);

        // Create a mock context for chain-A with a synthetic tendermint light client for chain-B
        let ctx_a = MockContext::new(
            ChainId::new("mockgaiaA".to_string(), 1),
            HostType::Mock,
            5,
            Height::new(1, 1).unwrap(),
        )
        .with_client_parametrized_with_chain_id(
            chain_id_b.clone(),
            &client_id,
            client_height,
            Some(tm_client_type()),
            Some(client_height),
        );

        // Create a mock context for chain-B
        let ctx_b = MockContext::new(
            chain_id_b.clone(),
            HostType::SyntheticTendermint,
            5,
            misbehaviour_height,
        );

        // Get chain-B's header at `misbehaviour_height`
        let header1: TmHeader = {
            let mut block = ctx_b.host_block(&misbehaviour_height).unwrap().clone();
            block.set_trusted_height(client_height);
            block.try_into_tm_block().unwrap().into()
        };

        // Generate an equivocal header for chain-B at `misbehaviour_height`
        let header2 = {
            let mut tm_block = HostBlock::generate_tm_block(
                chain_id_b,
                misbehaviour_height.revision_height(),
                Timestamp::now(),
            );
            tm_block.trusted_height = client_height;
            tm_block.into()
        };

        let msg = MsgSubmitMisbehaviour {
            client_id: client_id.clone(),
            misbehaviour: TmMisbehaviour::new(client_id.clone(), header1, header2)
                .unwrap()
                .into(),
            signer: get_dummy_account_id(),
        };

        let output = dispatch(&ctx_a, ClientMsg::Misbehaviour(msg));
        ensure_misbehaviour_result(output, &client_id, &tm_client_type());
    }

    #[test]
    fn test_misbehaviour_synthetic_tendermint_bft_time() {
        let client_id = ClientId::new(tm_client_type(), 0).unwrap();
        let client_height = Height::new(1, 20).unwrap();
        let misbehaviour_height = Height::new(1, 21).unwrap();
        let chain_id_b = ChainId::new("mockgaiaB".to_string(), 1);

        // Create a mock context for chain-A with a synthetic tendermint light client for chain-B
        let ctx_a = MockContext::new(
            ChainId::new("mockgaiaA".to_string(), 1),
            HostType::Mock,
            5,
            Height::new(1, 1).unwrap(),
        )
        .with_client_parametrized_with_chain_id(
            chain_id_b.clone(),
            &client_id,
            client_height,
            Some(tm_client_type()),
            Some(client_height),
        );

        // Generate `header1` for chain-B
        let header1 = {
            let mut tm_block = HostBlock::generate_tm_block(
                chain_id_b.clone(),
                misbehaviour_height.revision_height(),
                Timestamp::now(),
            );
            tm_block.trusted_height = client_height;
            tm_block
        };

        // Generate `header2` for chain-B which is identical to `header1` but with a conflicting
        // timestamp
        let header2 = {
            let timestamp =
                Timestamp::from_nanoseconds(Timestamp::now().nanoseconds() + 1_000_000_000)
                    .unwrap();
            let mut tm_block = HostBlock::generate_tm_block(
                chain_id_b,
                misbehaviour_height.revision_height(),
                timestamp,
            );
            tm_block.trusted_height = client_height;
            tm_block
        };

        let msg = MsgSubmitMisbehaviour {
            client_id: client_id.clone(),
            misbehaviour: TmMisbehaviour::new(client_id.clone(), header1.into(), header2.into())
                .unwrap()
                .into(),
            signer: get_dummy_account_id(),
        };

        let output = dispatch(&ctx_a, ClientMsg::Misbehaviour(msg));
        ensure_misbehaviour_result(output, &client_id, &tm_client_type());
    }
}
