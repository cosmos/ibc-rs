//! Protocol logic specific to processing ICS2 messages of type `MsgUpdateAnyClient`.

use tracing::debug;

use crate::prelude::*;

use crate::core::ics02_client::client_state::UpdatedState;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics02_client::events::UpdateClient;
use crate::core::ics02_client::msgs::update_client::MsgUpdateClient;
use crate::events::IbcEvent;

use crate::core::context::ContextError;

use crate::core::ics24_host::path::{ClientConsensusStatePath, ClientStatePath};

use crate::core::{ExecutionContext, ValidationContext};

pub(crate) fn validate<Ctx>(ctx: &Ctx, msg: MsgUpdateClient) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let MsgUpdateClient {
        client_id,
        header,
        signer: _,
    } = msg;

    // Read client type from the host chain store. The client should already exist.
    // Read client state from the host chain store.
    let client_state = ctx.client_state(&client_id)?;

    if client_state.is_frozen() {
        return Err(ClientError::ClientFrozen { client_id }.into());
    }

    // Read consensus state from the host chain store.
    let latest_client_cons_state_path =
        ClientConsensusStatePath::new(&client_id, &client_state.latest_height());
    let latest_consensus_state = ctx
        .consensus_state(&latest_client_cons_state_path)
        .map_err(|_| ClientError::ConsensusStateNotFound {
            client_id: client_id.clone(),
            height: client_state.latest_height(),
        })?;

    debug!("latest consensus state: {:?}", latest_consensus_state);

    let now = ctx.host_timestamp()?;
    let duration = now
        .duration_since(&latest_consensus_state.timestamp())
        .ok_or_else(|| ClientError::InvalidConsensusStateTimestamp {
            time1: latest_consensus_state.timestamp(),
            time2: now,
        })?;

    if client_state.expired(duration) {
        return Err(ClientError::HeaderNotWithinTrustPeriod {
            latest_time: latest_consensus_state.timestamp(),
            update_time: now,
        }
        .into());
    }

    let _ = client_state
        .check_header_and_update_state(ctx, client_id.clone(), header)
        .map_err(|e| ClientError::HeaderVerificationFailure {
            reason: e.to_string(),
        })?;

    Ok(())
}

pub(crate) fn execute<Ctx>(ctx: &mut Ctx, msg: MsgUpdateClient) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    let MsgUpdateClient {
        client_id,
        header,
        signer: _,
    } = msg;

    // Read client type from the host chain store. The client should already exist.
    // Read client state from the host chain store.
    let client_state = ctx.client_state(&client_id)?;

    let UpdatedState {
        client_state,
        consensus_state,
    } = client_state
        .check_header_and_update_state(ctx, client_id.clone(), header.clone())
        .map_err(|e| ClientError::HeaderVerificationFailure {
            reason: e.to_string(),
        })?;

    ctx.store_client_state(ClientStatePath::new(&client_id), client_state.clone())?;
    ctx.store_consensus_state(
        ClientConsensusStatePath::new(&client_id, &client_state.latest_height()),
        consensus_state,
    )?;
    ctx.store_update_time(
        client_id.clone(),
        client_state.latest_height(),
        ctx.host_timestamp()?,
    )?;
    ctx.store_update_height(
        client_id.clone(),
        client_state.latest_height(),
        ctx.host_height()?,
    )?;

    {
        let consensus_height = client_state.latest_height();

        ctx.emit_ibc_event(IbcEvent::UpdateClient(UpdateClient::new(
            client_id,
            client_state.client_type(),
            consensus_height,
            vec![consensus_height],
            header,
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;
    use ibc_proto::google::protobuf::Any;
    use test_log::test;

    use crate::clients::ics07_tendermint::client_type as tm_client_type;
    use crate::clients::ics07_tendermint::consensus_state::ConsensusState as TmConsensusState;
    use crate::core::ics02_client::client_state::ClientState;
    use crate::core::ics02_client::consensus_state::downcast_consensus_state;
    use crate::core::ics02_client::handler::update_client::{execute, validate};
    use crate::core::ics02_client::msgs::update_client::MsgUpdateClient;
    use crate::core::ics24_host::identifier::{ChainId, ClientId};
    use crate::core::ValidationContext;
    use crate::events::IbcEvent;
    use crate::mock::client_state::client_type as mock_client_type;
    use crate::mock::client_state::MockClientState;
    use crate::mock::context::MockContext;
    use crate::mock::header::MockHeader;
    use crate::mock::host::{HostBlock, HostType};
    use crate::test_utils::get_dummy_account_id;
    use crate::timestamp::Timestamp;
    use crate::Height;
    use crate::{downcast, prelude::*};

    #[test]
    fn test_update_client_ok() {
        let client_id = ClientId::default();
        let signer = get_dummy_account_id();

        let timestamp = Timestamp::now();

        let mut ctx = MockContext::default().with_client(&client_id, Height::new(0, 42).unwrap());
        let height = Height::new(0, 46).unwrap();
        let msg = MsgUpdateClient {
            client_id,
            header: MockHeader::new(height).with_timestamp(timestamp).into(),
            signer,
        };

        let res = validate(&ctx, msg.clone());

        assert!(res.is_ok(), "validation happy path");

        let res = execute(&mut ctx, msg.clone());
        assert!(res.is_ok(), "execution happy path");

        assert_eq!(
            ctx.client_state(&msg.client_id).unwrap(),
            MockClientState::new(MockHeader::new(height).with_timestamp(timestamp)).into_box()
        );
    }

    #[test]
    fn test_update_nonexisting_client() {
        let client_id = ClientId::from_str("mockclient1").unwrap();
        let signer = get_dummy_account_id();

        let ctx = MockContext::default().with_client(&client_id, Height::new(0, 42).unwrap());

        let msg = MsgUpdateClient {
            client_id: ClientId::from_str("nonexistingclient").unwrap(),
            header: MockHeader::new(Height::new(0, 46).unwrap()).into(),
            signer,
        };

        let res = validate(&ctx, msg);

        assert!(res.is_err());
    }

    #[test]
    fn test_update_synthetic_tendermint_client_adjacent_ok() {
        let client_id = ClientId::new(tm_client_type(), 0).unwrap();
        let client_height = Height::new(1, 20).unwrap();
        let update_height = Height::new(1, 21).unwrap();
        let chain_id_b = ChainId::new("mockgaiaB".to_string(), 1);

        let mut ctx = MockContext::new(
            ChainId::new("mockgaiaA".to_string(), 1),
            HostType::Mock,
            5,
            Height::new(1, 1).unwrap(),
        )
        .with_client_parametrized_with_chain_id(
            chain_id_b.clone(),
            &client_id,
            client_height,
            Some(tm_client_type()), // The target host chain (B) is synthetic TM.
            Some(client_height),
        );

        let ctx_b = MockContext::new(chain_id_b, HostType::SyntheticTendermint, 5, update_height);

        let signer = get_dummy_account_id();

        let mut block = ctx_b.host_block(&update_height).unwrap().clone();
        block.set_trusted_height(client_height);

        let latest_header_height = block.height();
        let msg = MsgUpdateClient {
            client_id,
            header: block.into(),
            signer,
        };

        let res = validate(&ctx, msg.clone());
        assert!(res.is_ok());

        let res = execute(&mut ctx, msg.clone());
        assert!(res.is_ok(), "result: {res:?}");

        let client_state = ctx.client_state(&msg.client_id).unwrap();
        assert!(!client_state.is_frozen());
        assert_eq!(client_state.latest_height(), latest_header_height);
    }

    #[test]
    fn test_update_synthetic_tendermint_client_non_adjacent_ok() {
        let client_id = ClientId::new(tm_client_type(), 0).unwrap();
        let client_height = Height::new(1, 20).unwrap();
        let update_height = Height::new(1, 21).unwrap();
        let chain_id_b = ChainId::new("mockgaiaB".to_string(), 1);

        let mut ctx = MockContext::new(
            ChainId::new("mockgaiaA".to_string(), 1),
            HostType::Mock,
            5,
            Height::new(1, 1).unwrap(),
        )
        .with_client_parametrized_history_with_chain_id(
            chain_id_b.clone(),
            &client_id,
            client_height,
            Some(tm_client_type()), // The target host chain (B) is synthetic TM.
            Some(client_height),
        );

        let ctx_b = MockContext::new(chain_id_b, HostType::SyntheticTendermint, 5, update_height);

        let signer = get_dummy_account_id();

        let mut block = ctx_b.host_block(&update_height).unwrap().clone();
        let trusted_height = client_height.clone().sub(1).unwrap();
        block.set_trusted_height(trusted_height);

        let latest_header_height = block.height();
        let msg = MsgUpdateClient {
            client_id,
            header: block.into(),
            signer,
        };

        let res = validate(&ctx, msg.clone());
        assert!(res.is_ok());

        let res = execute(&mut ctx, msg.clone());
        assert!(res.is_ok(), "result: {res:?}");

        let client_state = ctx.client_state(&msg.client_id).unwrap();
        assert!(!client_state.is_frozen());
        assert_eq!(client_state.latest_height(), latest_header_height);
    }

    #[test]
    fn test_update_synthetic_tendermint_client_duplicate_ok() {
        let client_id = ClientId::new(tm_client_type(), 0).unwrap();
        let client_height = Height::new(1, 20).unwrap();

        let chain_start_height = Height::new(1, 11).unwrap();

        let mut ctx = MockContext::new(
            ChainId::new("mockgaiaA".to_string(), 1),
            HostType::Mock,
            5,
            chain_start_height,
        )
        .with_client_parametrized(
            &client_id,
            client_height,
            Some(tm_client_type()), // The target host chain (B) is synthetic TM.
            Some(client_height),
        );

        let ctx_b = MockContext::new(
            ChainId::new("mockgaiaB".to_string(), 1),
            HostType::SyntheticTendermint,
            5,
            client_height,
        );

        let signer = get_dummy_account_id();

        let block = ctx_b.host_block(&client_height).unwrap().clone();
        let block = match block {
            HostBlock::SyntheticTendermint(mut theader) => {
                let cons_state = ctx.latest_consensus_states(&client_id, &client_height);
                if let Some(tcs) = downcast_consensus_state::<TmConsensusState>(cons_state.as_ref())
                {
                    theader.light_block.signed_header.header.time = tcs.timestamp;
                    theader.trusted_height = Height::new(1, 11).unwrap();
                }
                HostBlock::SyntheticTendermint(theader)
            }
            _ => block,
        };

        let latest_header_height = block.height();
        let msg = MsgUpdateClient {
            client_id,
            header: block.into(),
            signer,
        };

        let res = validate(&ctx, msg.clone());
        assert!(res.is_ok());

        let res = execute(&mut ctx, msg.clone());
        assert!(res.is_ok(), "result: {res:?}");

        let client_state = ctx.client_state(&msg.client_id).unwrap();
        assert!(!client_state.is_frozen());
        assert_eq!(client_state.latest_height(), latest_header_height);
        assert_eq!(client_state, ctx.latest_client_states(&msg.client_id));
    }

    #[test]
    fn test_update_synthetic_tendermint_client_lower_height() {
        let client_id = ClientId::new(tm_client_type(), 0).unwrap();
        let client_height = Height::new(1, 20).unwrap();

        let client_update_height = Height::new(1, 19).unwrap();

        let chain_start_height = Height::new(1, 11).unwrap();

        let ctx = MockContext::new(
            ChainId::new("mockgaiaA".to_string(), 1),
            HostType::Mock,
            5,
            chain_start_height,
        )
        .with_client_parametrized(
            &client_id,
            client_height,
            Some(tm_client_type()), // The target host chain (B) is synthetic TM.
            Some(client_height),
        );

        let ctx_b = MockContext::new(
            ChainId::new("mockgaiaB".to_string(), 1),
            HostType::SyntheticTendermint,
            5,
            client_height,
        );

        let signer = get_dummy_account_id();

        let block_ref = ctx_b.host_block(&client_update_height).unwrap();

        let msg = MsgUpdateClient {
            client_id,
            header: block_ref.clone().into(),
            signer,
        };

        let res = validate(&ctx, msg);
        assert!(res.is_err());
    }

    #[test]
    fn test_update_client_events() {
        let client_id = ClientId::default();
        let signer = get_dummy_account_id();

        let timestamp = Timestamp::now();

        let mut ctx = MockContext::default().with_client(&client_id, Height::new(0, 42).unwrap());
        let height = Height::new(0, 46).unwrap();
        let header: Any = MockHeader::new(height).with_timestamp(timestamp).into();
        let msg = MsgUpdateClient {
            client_id: client_id.clone(),
            header: header.clone(),
            signer,
        };

        let res = execute(&mut ctx, msg);
        assert!(res.is_ok());

        let update_client_event =
            downcast!(ctx.events.first().unwrap() => IbcEvent::UpdateClient).unwrap();

        assert_eq!(update_client_event.client_id(), &client_id);
        assert_eq!(update_client_event.client_type(), &mock_client_type());
        assert_eq!(update_client_event.consensus_height(), &height);
        assert_eq!(update_client_event.consensus_heights(), &vec![height]);
        assert_eq!(update_client_event.header(), &header);
    }
}
