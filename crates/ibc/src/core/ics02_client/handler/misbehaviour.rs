//! Protocol logic specific to processing ICS2 messages of type `MsgSubmitMisbehaviour`.
//!
use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics02_client::context::ClientReader;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics02_client::events::ClientMisbehaviour;
use crate::core::ics02_client::handler::ClientResult;
use crate::core::ics02_client::msgs::misbehaviour::MsgSubmitMisbehaviour;
use crate::core::ics24_host::identifier::ClientId;
use crate::events::IbcEvent;
use crate::handler::{HandlerOutput, HandlerResult};
use crate::prelude::*;

/// The result following the successful processing of a `MsgSubmitMisbehaviour` message.
#[derive(Clone, Debug, PartialEq)]
pub struct MisbehaviourResult {
    pub client_id: ClientId,
    pub client_state: Box<dyn ClientState>,
}

pub fn process(
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

// TODO(hu55a1n1): Add tests
