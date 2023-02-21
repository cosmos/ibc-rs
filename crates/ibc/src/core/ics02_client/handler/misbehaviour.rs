//! Protocol logic specific to processing ICS2 messages of type `MsgSubmitMisbehaviour`.
//!
use crate::prelude::*;

use crate::core::ics02_client::error::ClientError;
use crate::core::ics02_client::events::ClientMisbehaviour;
use crate::core::ics02_client::msgs::misbehaviour::MsgSubmitMisbehaviour;
use crate::events::IbcEvent;

use crate::core::ics24_host::path::ClientStatePath;

use crate::core::{ContextError, ExecutionContext, ValidationContext};

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
        .check_misbehaviour_and_update_state(ctx, client_id.clone(), misbehaviour)
        .map_err(|e| ClientError::MisbehaviourHandlingFailure {
            reason: e.to_string(),
        })?;

    Ok(())
}

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
        .check_misbehaviour_and_update_state(ctx, client_id.clone(), misbehaviour)
        .map_err(|e| ClientError::MisbehaviourHandlingFailure {
            reason: e.to_string(),
        })?;

    ctx.emit_ibc_event(IbcEvent::ClientMisbehaviour(ClientMisbehaviour::new(
        client_id.clone(),
        client_state.client_type(),
    )));

    ctx.store_client_state(ClientStatePath::new(&client_id), client_state)
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;
    use test_log::test;

    use crate::clients::ics07_tendermint::client_type as tm_client_type;
    use crate::clients::ics07_tendermint::header::Header as TmHeader;
    use crate::clients::ics07_tendermint::misbehaviour::Misbehaviour as TmMisbehaviour;
    use crate::core::ics02_client::client_type::ClientType;
    use crate::core::ics02_client::handler::misbehaviour::execute;
    use crate::core::ics02_client::handler::misbehaviour::validate;
    use crate::core::ics02_client::msgs::misbehaviour::MsgSubmitMisbehaviour;
    use crate::core::ics24_host::identifier::{ChainId, ClientId};
    use crate::core::ValidationContext;
    use crate::events::IbcEvent;
    use crate::mock::client_state::client_type as mock_client_type;
    use crate::mock::context::MockContext;
    use crate::mock::header::MockHeader;
    use crate::mock::host::{HostBlock, HostType};
    use crate::mock::misbehaviour::Misbehaviour as MockMisbehaviour;
    use crate::test_utils::get_dummy_account_id;
    use crate::timestamp::Timestamp;
    use crate::Height;
    use crate::{downcast, prelude::*};

    fn ensure_misbehaviour(ctx: &MockContext, client_id: &ClientId, client_type: &ClientType) {
        let client_state = ctx.client_state(client_id).unwrap();

        assert!(client_state.is_frozen());
        assert_eq!(
            client_state.frozen_height(),
            Some(Height::new(0, 1).unwrap())
        );

        // check events
        let misbehaviour_client_event =
            downcast!(ctx.events.first().unwrap() => IbcEvent::ClientMisbehaviour).unwrap();
        assert_eq!(ctx.events.len(), 1);
        assert_eq!(misbehaviour_client_event.client_id(), client_id);
        assert_eq!(misbehaviour_client_event.client_type(), client_type);
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

        let mut ctx = MockContext::default().with_client(&client_id, Height::new(0, 42).unwrap());

        let res = validate(&ctx, msg.clone());
        assert!(res.is_ok());
        let res = execute(&mut ctx, msg);
        assert!(res.is_ok());

        ensure_misbehaviour(&ctx, &client_id, &mock_client_type());
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
        let res = validate(&ctx, msg);
        assert!(res.is_err());
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
        let mut ctx_a = MockContext::new(
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

        let res = validate(&ctx_a, msg.clone());
        assert!(res.is_ok());
        let res = execute(&mut ctx_a, msg);
        assert!(res.is_ok());
        ensure_misbehaviour(&ctx_a, &client_id, &tm_client_type());
    }

    #[test]
    fn test_misbehaviour_synthetic_tendermint_bft_time() {
        let client_id = ClientId::new(tm_client_type(), 0).unwrap();
        let client_height = Height::new(1, 20).unwrap();
        let misbehaviour_height = Height::new(1, 21).unwrap();
        let chain_id_b = ChainId::new("mockgaiaB".to_string(), 1);

        // Create a mock context for chain-A with a synthetic tendermint light client for chain-B
        let mut ctx_a = MockContext::new(
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

        let res = validate(&ctx_a, msg.clone());
        assert!(res.is_ok());
        let res = execute(&mut ctx_a, msg);
        assert!(res.is_ok());
        ensure_misbehaviour(&ctx_a, &client_id, &tm_client_type());
    }
}
