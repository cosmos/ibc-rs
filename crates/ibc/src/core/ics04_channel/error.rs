use super::packet::Sequence;
use super::timeout::TimeoutHeight;
use crate::core::ics02_client::error as client_error;
use crate::core::ics03_connection::error as connection_error;
use crate::core::ics04_channel::channel::State;
use crate::core::ics05_port::error as port_error;
use crate::core::ics24_host::error::ValidationError;
use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
use crate::prelude::*;
use crate::proofs::ProofError;
use crate::signer::SignerError;
use crate::timestamp::Timestamp;
use crate::Height;

use displaydoc::Display;
use ibc_proto::protobuf::Error as TendermintError;

#[derive(Debug, Display)]
pub enum Error {
    /// ics03 connection error(`{0}`)
    Ics03Connection(connection_error::Error),
    /// ics05 port error(`{0}`)
    Ics05Port(port_error::Error),
    /// channel state unknown: `{state}`
    UnknownState { state: i32 },
    /// identifier error(`{0}`)
    Identifier(ValidationError),
    /// channel order type unknown: `{type_id}`
    UnknownOrderType { type_id: String },
    /// invalid connection hops length: expected `{expected}`; actual `{actual}`
    InvalidConnectionHopsLength { expected: usize, actual: usize },
    /// packet destination port `{port_id}` and channel `{channel_id}` doesn't match the counterparty's port/channel
    InvalidPacketCounterparty {
        port_id: PortId,
        channel_id: ChannelId,
    },
    /// invalid version, error(`{0}`)
    InvalidVersion(TendermintError),
    /// invalid signer address, error(`{0}`)
    Signer(SignerError),
    /// invalid proof, error(`{0}`)
    InvalidProof(ProofError),
    /// invalid proof: missing height
    MissingHeight,
    /// Missing sequence number for receiving packets on port `{port_id}` and channel `{channel_id}`
    MissingNextRecvSeq {
        port_id: PortId,
        channel_id: ChannelId,
    },
    /// packet sequence cannot be 0
    ZeroPacketSequence,
    /// packet data bytes cannot be empty
    ZeroPacketData,
    /// packet data bytes must be valid UTF-8 (this restriction will be lifted in the future)
    NonUtf8PacketData,
    /// invalid timeout height for the packet
    InvalidTimeoutHeight,
    /// invalid packet
    InvalidPacket,
    /// there is no packet in this message
    MissingPacket,
    /// missing channel id
    MissingChannelId,
    /// missing counterparty
    MissingCounterparty,
    /// no commong version
    NoCommonVersion,
    /// missing channel end
    MissingChannel,
    /// single version must be negociated on connection before opening channel
    InvalidVersionLengthConnection,
    /// the channel ordering is not supported by connection
    ChannelFeatureNotSuportedByConnection,
    /// the channel end (`{port_id}`, `{channel_id}`) does not exist
    ChannelNotFound {
        port_id: PortId,
        channel_id: ChannelId,
    },
    /// a different channel exists (was initialized) already for the same channel identifier `{channel_id}`
    ChannelMismatch { channel_id: ChannelId },
    /// the associated connection `{connection_id}` is not OPEN
    ConnectionNotOpen { connection_id: ConnectionId },
    /// Undefined counterparty connection for `{connection_id}`
    UndefinedConnectionCounterparty { connection_id: ConnectionId },
    /// Verification fails for the packet with the sequence number `{sequence}`, error(`{ics02_error}`)
    PacketVerificationFailed {
        sequence: Sequence,
        ics02_error: client_error::Error,
    },
    /// Error verifying channel state, error(`{0}`)
    VerifyChannelFailed(client_error::Error),
    /// Acknowledgment cannot be empty
    InvalidAcknowledgement,
    /// Packet acknowledgement exists for the packet with the sequence `{sequence}`
    AcknowledgementExists { sequence: Sequence },
    /// Missing sequence number for sending packets on port `{port_id}` and channel `{channel_id}`
    MissingNextSendSeq {
        port_id: PortId,
        channel_id: ChannelId,
    },
    /// String `{value}` cannot be converted to packet sequence, error(`{error}`)
    InvalidStringAsSequence {
        value: String,
        error: core::num::ParseIntError,
    },
    /// Invalid packet sequence `{given_sequence}` ≠ next send sequence `{next_sequence}`
    InvalidPacketSequence {
        given_sequence: Sequence,
        next_sequence: Sequence,
    },
    /// Receiving chain block height `{chain_height}` >= packet timeout height `{timeout_height}`
    LowPacketHeight {
        chain_height: Height,
        timeout_height: TimeoutHeight,
    },
    /// Packet timeout height `{timeout_height}` > chain height `{chain_height}`
    PacketTimeoutHeightNotReached {
        timeout_height: TimeoutHeight,
        chain_height: Height,
    },
    /// Packet timeout timestamp `{timeout_timestamp}` > chain timestamp `{chain_timestamp}`
    PacketTimeoutTimestampNotReached {
        timeout_timestamp: Timestamp,
        chain_timestamp: Timestamp,
    },
    /// Receiving chain block timestamp >= packet timeout timestamp
    LowPacketTimestamp,
    /// Invalid packet timeout timestamp value, error(`{0}`)
    InvalidPacketTimestamp(crate::timestamp::ParseTimestampError),
    /// Invalid timestamp in consensus state; timestamp must be a positive value
    ErrorInvalidConsensusState,
    /// Client with id `{client_id}` is frozen
    FrozenClient { client_id: ClientId },
    /// Invalid channel id in counterparty
    InvalidCounterpartyChannelId,
    /// Channel `{channel_id}` should not be state `{state}`
    InvalidChannelState { channel_id: ChannelId, state: State },
    /// Channel `{channel_id}` is Closed
    ChannelClosed { channel_id: ChannelId },
    /// Handshake proof verification fails at ChannelOpenAck
    ChanOpenAckProofVerification,
    /// Commitment for the packet `{sequence}` not found
    PacketCommitmentNotFound { sequence: Sequence },
    /// The stored commitment of the packet `{sequence}` is incorrect
    IncorrectPacketCommitment { sequence: Sequence },
    /// Receipt for the packet `{sequence}` not found
    PacketReceiptNotFound { sequence: Sequence },
    /// Acknowledgment for the packet `{sequence}` not found
    PacketAcknowledgementNotFound { sequence: Sequence },
    /// Missing sequence number for ack packets on port `{port_id}` and channel `{channel_id}`
    MissingNextAckSeq {
        port_id: PortId,
        channel_id: ChannelId,
    },
    /// Processed time for the client `{client_id}` at height `{height}` not found
    ProcessedTimeNotFound { client_id: ClientId, height: Height },
    /// Processed height for the client `{client_id}` at height `{height}` not found
    ProcessedHeightNotFound { client_id: ClientId, height: Height },
    /// route not found
    RouteNotFound,
    /// implementation specific error
    ImplementationSpecific,
    /// application module error: `{description}`
    AppModule { description: String },
    /// Failed to convert abci event to IbcEvent: `{abci_event}`
    AbciConversionFailed { abci_event: String },
    /// other error: `{description}`
    Other { description: String },
}

// impl Error {
//     pub fn chan_open_confirm_proof_verification(e: Error) -> Error {
//         e.add_trace(&"Handshake proof verification fails at ChannelOpenConfirm")
//     }
// }
