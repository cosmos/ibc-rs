//! This module implements the processing logic for ICS3 (connection open
//! handshake) messages.
//!
//! Naming is hard in the connection handshakes, since we deal with a client on
//! a *counterparty* chain, which is itself a light client of *self* (the chain
//! the handler is currently running on). So depending on the frame of reference
//! chosen, e.g. counterparty_client_state could mean:
//!
//! 1. the client state of the client running on the counterparty chain
//! 2. or the state of the "counterparty client" (that is, the client that we
//!    run of the counterparty chain) running on the host chain
//!
//! We remove such ambiguity by adopting the following conventions:
//! + we call "chain A" the chain that runs `ConnOpenInit` and `ConnOpenAck`
//! + we call "chain B" is the chain that runs `ConnOpenTry` and `ConnOpenConfirm`
//! + In variable names,
//!     + `on_a` implies "stored on chain A"
//!     + `of_a` implies "of light client for chain A"
//! So `client_state_of_a_on_b` means "the client state of light client for chain A stored on chain B"

use crate::core::ics03_connection::connection::ConnectionEnd;
use crate::core::ics03_connection::context::ConnectionReader;
use crate::core::ics03_connection::error::Error;
use crate::core::ics03_connection::msgs::ConnectionMsg;
use crate::core::ics24_host::identifier::ConnectionId;
use crate::handler::HandlerOutput;

pub mod conn_open_ack;
pub mod conn_open_confirm;
pub mod conn_open_init;
pub mod conn_open_try;

/// Defines the possible states of a connection identifier in a `ConnectionResult`.
#[derive(Clone, Debug)]
pub enum ConnectionIdState {
    /// Specifies that the handler allocated a new connection identifier. This happens during the
    /// processing of either the `MsgConnectionOpenInit` or `MsgConnectionOpenTry` message.
    Generated,

    /// Specifies that the handler reused a previously-allocated connection identifier.
    Reused,
}

#[derive(Clone, Debug)]
pub struct ConnectionResult {
    /// The identifier for the connection which the handler processed. Typically this represents the
    /// newly-generated connection id (e.g., when processing `MsgConnectionOpenInit`) or
    /// an existing connection id (e.g., for `MsgConnectionOpenAck`).
    pub connection_id: ConnectionId,

    /// The state of the connection identifier (whether it was newly-generated or not).
    pub connection_id_state: ConnectionIdState,

    /// The connection end, which the handler produced as a result of processing the message.
    pub connection_end: ConnectionEnd,
}

/// General entry point for processing any type of message related to the ICS3 connection open
/// handshake protocol.
pub fn dispatch<Ctx>(
    ctx: &Ctx,
    msg: ConnectionMsg,
) -> Result<HandlerOutput<ConnectionResult>, Error>
where
    Ctx: ConnectionReader,
{
    match msg {
        ConnectionMsg::ConnectionOpenInit(msg) => conn_open_init::process(ctx, msg),
        ConnectionMsg::ConnectionOpenTry(msg) => conn_open_try::process(ctx, *msg),
        ConnectionMsg::ConnectionOpenAck(msg) => conn_open_ack::process(ctx, *msg),
        ConnectionMsg::ConnectionOpenConfirm(msg) => conn_open_confirm::process(ctx, msg),
    }
}
