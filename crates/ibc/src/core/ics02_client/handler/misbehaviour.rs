//! Protocol logic specific to processing ICS2 messages of type `MsgUpgradeAnyClient`.
//!
use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics02_client::context::ClientReader;
use crate::core::ics02_client::error::Error;
use crate::core::ics02_client::events::ClientMisbehaviour;
use crate::core::ics02_client::handler::ClientResult;
use crate::core::ics02_client::msgs::misbehaviour::MsgSubmitMisbehaviour;
use crate::core::ics24_host::identifier::ClientId;
use crate::events::IbcEvent;
use crate::handler::{HandlerOutput, HandlerResult};
use crate::prelude::*;

/// The result following the successful processing of a `MsgSubmitMisbehaviour` message.
/// This data type should be used with a qualified name `misbehaviour::Result` to avoid ambiguity.
#[derive(Clone, Debug, PartialEq)]
pub struct Result {
    pub client_id: ClientId,
    pub client_state: Box<dyn ClientState>,
}

pub fn process(
    ctx: &dyn ClientReader,
    msg: MsgSubmitMisbehaviour,
) -> HandlerResult<ClientResult, Error> {
    let mut output = HandlerOutput::builder();

    let MsgSubmitMisbehaviour {
        client_id,
        misbehaviour,
        signer: _,
    } = msg;

    // Read client state from the host chain store.
    let client_state = ctx.client_state(&client_id)?;

    if client_state.is_frozen() {
        return Err(Error::client_frozen(client_id));
    }

    // Read consensus state from the host chain store.
    let latest_consensus_state = ctx
        .consensus_state(&client_id, client_state.latest_height())
        .map_err(|_| {
            Error::consensus_state_not_found(client_id.clone(), client_state.latest_height())
        })?;

    let now = ClientReader::host_timestamp(ctx)?;
    let duration = now
        .duration_since(&latest_consensus_state.timestamp())
        .ok_or_else(|| {
            Error::invalid_consensus_state_timestamp(latest_consensus_state.timestamp(), now)
        })?;

    if client_state.expired(duration) {
        return Err(Error::client_expired(
            client_id,
            latest_consensus_state.timestamp(),
            now,
        ));
    }

    let client_state = client_state
        .check_misbehaviour_and_update_state(ctx, client_id.clone(), misbehaviour)
        .map_err(|e| Error::misbehaviour_handling_failure(e.to_string()))?;

    output.emit(IbcEvent::ClientMisbehaviour(ClientMisbehaviour::new(
        client_id.clone(),
        client_state.client_type(),
    )));

    let result = ClientResult::Misbehaviour(Result {
        client_id,
        client_state,
    });

    Ok(output.with_result(result))
}

// TODO(hu55a1n1): Add tests
