//! Message definitions for the connection handshake datagrams.
//!
//! We define each of the four messages in the connection handshake protocol as a `struct`.
//! Each such message comprises the same fields as the datagrams defined in ICS3 English spec:
//! <https://github.com/cosmos/ibc/tree/master/spec/core/ics-003-connection-semantics>.
//!
//! One departure from ICS3 is that we abstract the three counterparty fields (connection id,
//! prefix, and client id) into a single field of type `Counterparty`; this applies to messages
//! `MsgConnectionOpenInit` and `MsgConnectionOpenTry`. One other difference with regards to
//! abstraction is that all proof-related attributes in a message are encapsulated in `Proofs` type.
//!
//! Another difference to ICS3 specs is that each message comprises an additional field called
//! `signer` which is specific to Cosmos-SDK.

use ibc_primitives::prelude::*;

mod conn_open_ack;
mod conn_open_confirm;
mod conn_open_init;
mod conn_open_try;

pub use conn_open_ack::*;
pub use conn_open_confirm::*;
pub use conn_open_init::*;
pub use conn_open_try::*;

/// Enumeration of all possible messages that the ICS3 protocol processes.
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, derive_more::From)]
pub enum ConnectionMsg {
    OpenInit(MsgConnectionOpenInit),
    OpenTry(MsgConnectionOpenTry),
    OpenAck(MsgConnectionOpenAck),
    OpenConfirm(MsgConnectionOpenConfirm),
}
