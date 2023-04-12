//! Protocol logic specific to processing ICS2 messages of type `MsgCreateClient`.

use crate::prelude::*;

use crate::core::context::ContextError;

use crate::core::ics24_host::path::ClientConsensusStatePath;

use crate::core::ics24_host::path::ClientStatePath;

use crate::core::ics24_host::path::ClientTypePath;

use crate::core::ExecutionContext;

use crate::core::ValidationContext;

use crate::core::ics02_client::error::ClientError;
use crate::core::ics02_client::events::CreateClient;
use crate::core::ics02_client::msgs::create_client::MsgCreateClient;
use crate::core::ics24_host::identifier::ClientId;
use crate::events::IbcEvent;

pub(crate) fn validate<Ctx>(ctx: &Ctx, msg: MsgCreateClient) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let MsgCreateClient {
        client_state,
        consensus_state: _,
        signer: _,
    } = msg;

    // Construct this client's identifier
    let id_counter = ctx.client_counter()?;

    let client_state = ctx.decode_client_state(client_state)?;

    let client_type = client_state.client_type();

    let client_id = ClientId::new(client_type, id_counter).map_err(|e| {
        ClientError::ClientIdentifierConstructor {
            client_type: client_state.client_type(),
            counter: id_counter,
            validation_error: e,
        }
    })?;

    if ctx.client_state(&client_id).is_ok() {
        return Err(ClientError::ClientStateAlreadyExists { client_id }.into());
    };

    Ok(())
}

pub(crate) fn execute<Ctx>(ctx: &mut Ctx, msg: MsgCreateClient) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    let MsgCreateClient {
        client_state,
        consensus_state,
        signer: _,
    } = msg;

    // Construct this client's identifier
    let id_counter = ctx.client_counter()?;

    let client_state = ctx.decode_client_state(client_state)?;

    let client_type = client_state.client_type();

    let client_id = ClientId::new(client_type.clone(), id_counter).map_err(|e| {
        ContextError::from(ClientError::ClientIdentifierConstructor {
            client_type: client_state.client_type(),
            counter: id_counter,
            validation_error: e,
        })
    })?;
    let consensus_state = client_state.initialise(consensus_state)?;

    ctx.store_client_type(ClientTypePath::new(&client_id), client_type.clone())?;
    ctx.store_client_state(ClientStatePath::new(&client_id), client_state.clone())?;
    ctx.store_consensus_state(
        ClientConsensusStatePath::new(&client_id, &client_state.latest_height()),
        consensus_state,
    )?;
    ctx.increase_client_counter();
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

    let event = IbcEvent::CreateClient(CreateClient::new(
        client_id.clone(),
        client_type,
        client_state.latest_height(),
    ));
    ctx.emit_ibc_event(IbcEvent::Message(event.event_type()));
    ctx.emit_ibc_event(event);

    ctx.log_message(format!(
        "success: generated new client identifier: {client_id}"
    ));

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use test_log::test;

    use crate::clients::ics07_tendermint::client_state::ClientState as TmClientState;
    use crate::clients::ics07_tendermint::client_type as tm_client_type;
    use crate::clients::ics07_tendermint::consensus_state::ConsensusState as TmConsensusState;
    use crate::clients::ics07_tendermint::header::test_util::get_dummy_tendermint_header;
    use crate::core::ics02_client::handler::create_client::{execute, validate};
    use crate::core::ics02_client::msgs::create_client::MsgCreateClient;
    use crate::core::ics24_host::identifier::ClientId;
    use crate::core::ValidationContext;
    use crate::mock::client_state::{client_type as mock_client_type, MockClientState};
    use crate::mock::consensus_state::MockConsensusState;
    use crate::mock::context::MockContext;
    use crate::mock::header::MockHeader;
    use crate::test_utils::get_dummy_account_id;
    use crate::Height;

    #[test]
    fn test_create_client_ok() {
        let mut ctx = MockContext::default();
        let signer = get_dummy_account_id();
        let height = Height::new(0, 42).unwrap();

        let msg = MsgCreateClient::new(
            MockClientState::new(MockHeader::new(height)).into(),
            MockConsensusState::new(MockHeader::new(height)).into(),
            signer,
        );

        let client_type = mock_client_type();

        let client_id = {
            let id_counter = ctx.client_counter().unwrap();
            ClientId::new(client_type.clone(), id_counter).unwrap()
        };

        let res = validate(&ctx, msg.clone());

        assert!(res.is_ok(), "validation happy path");

        let res = execute(&mut ctx, msg.clone());

        assert!(res.is_ok(), "execution happy path");

        let expected_client_state = ctx.decode_client_state(msg.client_state).unwrap();
        assert_eq!(expected_client_state.client_type(), client_type);
        assert_eq!(ctx.client_state(&client_id).unwrap(), expected_client_state);
    }

    #[test]
    fn test_tm_create_client_ok() {
        let signer = get_dummy_account_id();

        let mut ctx = MockContext::default();

        let tm_header = get_dummy_tendermint_header();

        let tm_client_state = TmClientState::new_dummy_from_header(tm_header.clone()).into();

        let client_type = tm_client_type();

        let client_id = {
            let id_counter = ctx.client_counter().unwrap();
            ClientId::new(client_type.clone(), id_counter).unwrap()
        };

        let msg = MsgCreateClient::new(
            tm_client_state,
            TmConsensusState::try_from(tm_header).unwrap().into(),
            signer,
        );

        let res = validate(&ctx, msg.clone());
        assert!(res.is_ok(), "tendermint client validation happy path");

        let res = execute(&mut ctx, msg.clone());
        assert!(res.is_ok(), "tendermint client execution happy path");

        let expected_client_state = ctx.decode_client_state(msg.client_state).unwrap();
        assert_eq!(expected_client_state.client_type(), client_type);
        assert_eq!(ctx.client_state(&client_id).unwrap(), expected_client_state);
    }
}
