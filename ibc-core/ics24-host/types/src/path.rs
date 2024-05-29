//! Defines all store paths used by IBC

/// Path-space as listed in ICS-024
/// https://github.com/cosmos/ibc/tree/master/spec/core/ics-024-host-requirements#path-space
/// Some of these are implemented in other ICSs, but ICS-024 has a nice summary table.
///
use core::str::FromStr;

use derive_more::{Display, From};
use ibc_primitives::prelude::*;

use crate::identifiers::{ChannelId, ClientId, ConnectionId, PortId, Sequence};

pub const NEXT_CLIENT_SEQUENCE: &str = "nextClientSequence";
pub const NEXT_CONNECTION_SEQUENCE: &str = "nextConnectionSequence";
pub const NEXT_CHANNEL_SEQUENCE: &str = "nextChannelSequence";

pub const CLIENT_PREFIX: &str = "clients";
pub const CLIENT_STATE: &str = "clientState";
pub const CONSENSUS_STATE_PREFIX: &str = "consensusStates";
pub const CONNECTION_PREFIX: &str = "connections";
pub const CHANNEL_PREFIX: &str = "channels";
pub const CHANNEL_END_PREFIX: &str = "channelEnds";
pub const PORT_PREFIX: &str = "ports";
pub const SEQUENCE_PREFIX: &str = "sequences";
pub const NEXT_SEQ_SEND_PREFIX: &str = "nextSequenceSend";
pub const NEXT_SEQ_RECV_PREFIX: &str = "nextSequenceRecv";
pub const NEXT_SEQ_ACK_PREFIX: &str = "nextSequenceAck";
pub const PACKET_COMMITMENT_PREFIX: &str = "commitments";
pub const PACKET_ACK_PREFIX: &str = "acks";
pub const PACKET_RECEIPT_PREFIX: &str = "receipts";

pub const ITERATE_CONSENSUS_STATE_PREFIX: &str = "iterateConsensusStates";
pub const PROCESSED_TIME: &str = "processedTime";
pub const PROCESSED_HEIGHT: &str = "processedHeight";

/// ABCI client upgrade keys
/// - The key identifying the upgraded IBC state within the upgrade sub-store
pub const UPGRADED_IBC_STATE: &str = "upgradedIBCState";
/// - The key identifying the upgraded client state
pub const UPGRADED_CLIENT_STATE: &str = "upgradedClient";
/// - The key identifying the upgraded consensus state
pub const UPGRADED_CLIENT_CONSENSUS_STATE: &str = "upgradedConsState";

/// The Path enum abstracts out the different sub-paths.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, From, Display)]
pub enum Path {
    NextClientSequence(NextClientSequencePath),
    NextConnectionSequence(NextConnectionSequencePath),
    NextChannelSequence(NextChannelSequencePath),
    ClientState(ClientStatePath),
    ClientConsensusState(ClientConsensusStatePath),
    ClientUpdateTime(ClientUpdateTimePath),
    ClientUpdateHeight(ClientUpdateHeightPath),
    ClientConnection(ClientConnectionPath),
    Connection(ConnectionPath),
    Ports(PortPath),
    ChannelEnd(ChannelEndPath),
    SeqSend(SeqSendPath),
    SeqRecv(SeqRecvPath),
    SeqAck(SeqAckPath),
    Commitment(CommitmentPath),
    Ack(AckPath),
    Receipt(ReceiptPath),
    UpgradeClient(UpgradeClientPath),
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
#[display(fmt = "{NEXT_CLIENT_SEQUENCE}")]
pub struct NextClientSequencePath;

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
#[display(fmt = "{NEXT_CONNECTION_SEQUENCE}")]
pub struct NextConnectionSequencePath;

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
#[display(fmt = "{NEXT_CHANNEL_SEQUENCE}")]
pub struct NextChannelSequencePath;

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display, From)]
#[display(fmt = "{CLIENT_PREFIX}/{_0}/{CLIENT_STATE}")]
pub struct ClientStatePath(pub ClientId);

impl ClientStatePath {
    pub fn new(client_id: ClientId) -> ClientStatePath {
        ClientStatePath(client_id)
    }

    /// Returns the client store prefix under which all the client states are
    /// stored: "clients".
    pub fn prefix() -> String {
        CLIENT_PREFIX.to_string()
    }

    /// Returns the final part (leaf) of the path under which an individual
    /// client state is stored: "clientState".
    pub fn leaf() -> String {
        CLIENT_STATE.to_string()
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
#[display(
    fmt = "{CLIENT_PREFIX}/{client_id}/{CONSENSUS_STATE_PREFIX}/{revision_number}-{revision_height}"
)]
pub struct ClientConsensusStatePath {
    pub client_id: ClientId,
    pub revision_number: u64,
    pub revision_height: u64,
}

// Returns the full consensus state path of specific client in the format:
// "clients/{client_id}/consensusStates" as a string.
pub fn full_consensus_state_path(client_id: &ClientId) -> String {
    format!("{CLIENT_PREFIX}/{client_id}/{CONSENSUS_STATE_PREFIX}")
}

impl ClientConsensusStatePath {
    /// Constructs a new `ClientConsensusStatePath`.
    pub fn new(
        client_id: ClientId,
        revision_number: u64,
        revision_height: u64,
    ) -> ClientConsensusStatePath {
        ClientConsensusStatePath {
            client_id,
            revision_number,
            revision_height,
        }
    }

    /// Returns the path representing the parent group under which all consensus
    /// states are stored: "clients/{client_id}/consensusStates".
    pub fn parent(&self) -> String {
        full_consensus_state_path(&self.client_id)
    }

    /// Returns the final part (leaf) of the path under which an individual
    /// consensus state is stored:
    /// "consensusStates/{revision_number}-{revision_height}".
    pub fn leaf(&self) -> String {
        format!(
            "{CONSENSUS_STATE_PREFIX}/{}-{}",
            self.revision_number, self.revision_height
        )
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
#[display(
    fmt = "{CLIENT_PREFIX}/{client_id}/{CONSENSUS_STATE_PREFIX}/{revision_number}-{revision_height}/{PROCESSED_TIME}"
)]
pub struct ClientUpdateTimePath {
    pub client_id: ClientId,
    pub revision_number: u64,
    pub revision_height: u64,
}

impl ClientUpdateTimePath {
    /// Constructs a new `ClientUpdateTimePath`.
    pub fn new(client_id: ClientId, revision_number: u64, revision_height: u64) -> Self {
        Self {
            client_id,
            revision_number,
            revision_height,
        }
    }

    /// Returns the path representing the parent group under which all the
    /// processed times are stored: "clients/{client_id}/consensusStates".
    pub fn parent(&self) -> String {
        full_consensus_state_path(&self.client_id)
    }

    /// Returns the final part (leaf) of the path under which an individual
    /// processed time is stored:
    /// "consensusStates/{revision_number}-{revision_height}/processedTime".
    pub fn leaf(&self) -> String {
        format!(
            "{CONSENSUS_STATE_PREFIX}/{}-{}/{PROCESSED_TIME}",
            self.revision_number, self.revision_height
        )
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
#[display(
    fmt = "{CLIENT_PREFIX}/{client_id}/{CONSENSUS_STATE_PREFIX}/{revision_number}-{revision_height}/{PROCESSED_HEIGHT}"
)]
pub struct ClientUpdateHeightPath {
    pub client_id: ClientId,
    pub revision_number: u64,
    pub revision_height: u64,
}

impl ClientUpdateHeightPath {
    pub fn new(client_id: ClientId, revision_number: u64, revision_height: u64) -> Self {
        Self {
            client_id,
            revision_number,
            revision_height,
        }
    }

    /// Returns the path representing the parent group under which all the
    /// processed heights are stored: "clients/{client_id}/consensusStates".
    pub fn parent(&self) -> String {
        full_consensus_state_path(&self.client_id)
    }

    /// Returns the final part (leaf) of the path under which an individual
    /// processed height is stored:
    /// "consensusStates/{revision_number}-{revision_height}/processedHeight".
    pub fn leaf(&self) -> String {
        format!(
            "{CONSENSUS_STATE_PREFIX}/{}-{}/{PROCESSED_HEIGHT}",
            self.revision_number, self.revision_height
        )
    }
}

/// This iteration key is namely used by the `ibc-go` implementation as an
/// efficient approach for iterating over consensus states. This is specifically
/// incorporated to facilitate cross-compatibility with `ibc-go` when developing
/// CosmWasm-driven light clients with `ibc-rs`.
///
/// The key is formatted like so:
/// `iterateConsensusStates{BigEndianRevisionBytes}{BigEndianHeightBytes}`
/// to ensure that the lexicographic order of iteration keys match the
/// height order of the consensus states.
///
/// See `ibc-go`
/// [documentation](https://github.com/cosmos/ibc-go/blob/016a6095b577ecb9323edad508cff19d017636a1/modules/light-clients/07-tendermint/store.go#L19-L34)
/// for more details.
pub fn iteration_key(revision_number: u64, revision_height: u64) -> Vec<u8> {
    let mut path = Vec::new();
    path.extend_from_slice(ITERATE_CONSENSUS_STATE_PREFIX.as_bytes());
    path.extend(revision_number.to_be_bytes());
    path.extend(revision_height.to_be_bytes());
    path
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
#[display(fmt = "{CLIENT_PREFIX}/{_0}/{CONNECTION_PREFIX}")]
pub struct ClientConnectionPath(pub ClientId);

impl ClientConnectionPath {
    pub fn new(client_id: ClientId) -> ClientConnectionPath {
        ClientConnectionPath(client_id)
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
#[display(fmt = "{CONNECTION_PREFIX}/{_0}")]
pub struct ConnectionPath(pub ConnectionId);

impl ConnectionPath {
    pub fn new(connection_id: &ConnectionId) -> ConnectionPath {
        ConnectionPath(connection_id.clone())
    }

    /// Returns the connection store prefix under which all the connections are
    /// stored: "connections".
    pub fn prefix() -> String {
        CONNECTION_PREFIX.to_string()
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
#[display(fmt = "{PORT_PREFIX}/{_0}")]
pub struct PortPath(pub PortId);

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
#[display(fmt = "{CHANNEL_END_PREFIX}/{PORT_PREFIX}/{_0}/{CHANNEL_PREFIX}/{_1}")]
pub struct ChannelEndPath(pub PortId, pub ChannelId);

impl ChannelEndPath {
    pub fn new(port_id: &PortId, channel_id: &ChannelId) -> ChannelEndPath {
        ChannelEndPath(port_id.clone(), channel_id.clone())
    }

    /// Returns the channel end store prefix under which all the channel ends
    /// are stored: "channelEnds".
    pub fn prefix() -> String {
        CHANNEL_END_PREFIX.to_string()
    }

    /// Returns the parent group path under which all the sequences of a channel are
    /// stored with the format:
    /// "{prefix}/ports/{port_id}/channels/{channel_id}/sequences"
    fn full_sequences_path(&self, prefix: &str) -> String {
        format!(
            "{prefix}/{PORT_PREFIX}/{}/{CHANNEL_PREFIX}/{}/{SEQUENCE_PREFIX}",
            self.0, self.1,
        )
    }

    /// Returns the parent group path under which all the commitment packets of
    /// a channel are stored.
    pub fn commitments_path(&self) -> String {
        self.full_sequences_path(PACKET_COMMITMENT_PREFIX)
    }

    /// Returns the parent group path under which all the ack packets of a
    /// channel are stored.
    pub fn acks_path(&self) -> String {
        self.full_sequences_path(PACKET_ACK_PREFIX)
    }

    /// Returns the parent group path under which all the receipt packets of a
    /// channel are stored.
    pub fn receipts_path(&self) -> String {
        self.full_sequences_path(PACKET_RECEIPT_PREFIX)
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
#[display(fmt = "{NEXT_SEQ_SEND_PREFIX}/{PORT_PREFIX}/{_0}/{CHANNEL_PREFIX}/{_1}")]
pub struct SeqSendPath(pub PortId, pub ChannelId);

impl SeqSendPath {
    pub fn new(port_id: &PortId, channel_id: &ChannelId) -> SeqSendPath {
        SeqSendPath(port_id.clone(), channel_id.clone())
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
#[display(fmt = "{NEXT_SEQ_RECV_PREFIX}/{PORT_PREFIX}/{_0}/{CHANNEL_PREFIX}/{_1}")]
pub struct SeqRecvPath(pub PortId, pub ChannelId);

impl SeqRecvPath {
    pub fn new(port_id: &PortId, channel_id: &ChannelId) -> SeqRecvPath {
        SeqRecvPath(port_id.clone(), channel_id.clone())
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
#[display(fmt = "{NEXT_SEQ_ACK_PREFIX}/{PORT_PREFIX}/{_0}/{CHANNEL_PREFIX}/{_1}")]
pub struct SeqAckPath(pub PortId, pub ChannelId);

impl SeqAckPath {
    pub fn new(port_id: &PortId, channel_id: &ChannelId) -> SeqAckPath {
        SeqAckPath(port_id.clone(), channel_id.clone())
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
#[display(
    fmt = "{PACKET_COMMITMENT_PREFIX}/{PORT_PREFIX}/{port_id}/{CHANNEL_PREFIX}/{channel_id}/{SEQUENCE_PREFIX}/{sequence}"
)]
pub struct CommitmentPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: Sequence,
}

impl CommitmentPath {
    pub fn new(port_id: &PortId, channel_id: &ChannelId, sequence: Sequence) -> CommitmentPath {
        CommitmentPath {
            port_id: port_id.clone(),
            channel_id: channel_id.clone(),
            sequence,
        }
    }

    /// Returns the commitment store prefix under which all the packet
    /// commitments are stored: "commitments"
    pub fn prefix() -> String {
        PACKET_COMMITMENT_PREFIX.to_string()
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
#[display(
    fmt = "{PACKET_ACK_PREFIX}/{PORT_PREFIX}/{port_id}/{CHANNEL_PREFIX}/{channel_id}/{SEQUENCE_PREFIX}/{sequence}"
)]
pub struct AckPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: Sequence,
}

impl AckPath {
    pub fn new(port_id: &PortId, channel_id: &ChannelId, sequence: Sequence) -> AckPath {
        AckPath {
            port_id: port_id.clone(),
            channel_id: channel_id.clone(),
            sequence,
        }
    }

    /// Returns the ack store prefix under which all the packet acks are stored:
    /// "acks"
    pub fn prefix() -> String {
        PACKET_ACK_PREFIX.to_string()
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
#[display(
    fmt = "{PACKET_RECEIPT_PREFIX}/{PORT_PREFIX}/{port_id}/{CHANNEL_PREFIX}/{channel_id}/{SEQUENCE_PREFIX}/{sequence}"
)]
pub struct ReceiptPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: Sequence,
}

impl ReceiptPath {
    pub fn new(port_id: &PortId, channel_id: &ChannelId, sequence: Sequence) -> ReceiptPath {
        ReceiptPath {
            port_id: port_id.clone(),
            channel_id: channel_id.clone(),
            sequence,
        }
    }

    /// Returns the receipt store prefix under which all the packet receipts are
    /// stored: "receipts"
    pub fn prefix() -> String {
        PACKET_RECEIPT_PREFIX.to_string()
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Paths that are specific for client upgrades.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
pub enum UpgradeClientPath {
    #[display(fmt = "{UPGRADED_IBC_STATE}/{_0}/{UPGRADED_CLIENT_STATE}")]
    UpgradedClientState(u64),
    #[display(fmt = "{UPGRADED_IBC_STATE}/{_0}/{UPGRADED_CLIENT_CONSENSUS_STATE}")]
    UpgradedClientConsensusState(u64),
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Sub-paths which are not part of the specification, but are still
/// useful to represent for parsing purposes.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SubPath {
    Channels(ChannelId),
    Sequences(Sequence),
}

impl Path {
    /// Indication if the path is provable.
    pub fn is_provable(&self) -> bool {
        !matches!(&self, Path::ClientConnection(_) | Path::Ports(_))
    }

    /// into_bytes implementation
    pub fn into_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

#[derive(Debug, displaydoc::Display)]
pub enum PathError {
    /// `{path}` could not be parsed into a Path
    ParseFailure { path: String },
}

#[cfg(feature = "std")]
impl std::error::Error for PathError {}

/// The FromStr trait allows paths encoded as strings to be parsed into Paths.
impl FromStr for Path {
    type Err = PathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let components: Vec<&str> = s.split('/').collect();

        parse_next_sequence(&components)
            .or_else(|| parse_client_paths(&components))
            .or_else(|| parse_connections(&components))
            .or_else(|| parse_ports(&components))
            .or_else(|| parse_channel_ends(&components))
            .or_else(|| parse_seqs(&components))
            .or_else(|| parse_commitments(&components))
            .or_else(|| parse_acks(&components))
            .or_else(|| parse_receipts(&components))
            .or_else(|| parse_upgrades(&components))
            .ok_or(PathError::ParseFailure {
                path: s.to_string(),
            })
    }
}

fn parse_next_sequence(components: &[&str]) -> Option<Path> {
    if components.len() != 1 {
        return None;
    }

    match *components.first()? {
        NEXT_CLIENT_SEQUENCE => Some(NextClientSequencePath.into()),
        NEXT_CONNECTION_SEQUENCE => Some(NextConnectionSequencePath.into()),
        NEXT_CHANNEL_SEQUENCE => Some(NextChannelSequencePath.into()),
        _ => None,
    }
}

fn parse_client_paths(components: &[&str]) -> Option<Path> {
    let first = *components.first()?;

    if first != CLIENT_PREFIX {
        return None;
    }

    let client_id = ClientId::from_str(components[1]).ok()?;

    if components.len() == 3 {
        match components[2] {
            CLIENT_STATE => Some(ClientStatePath(client_id).into()),
            CONNECTION_PREFIX => Some(ClientConnectionPath(client_id).into()),
            _ => None,
        }
    } else if components.len() == 4 || components.len() == 5 {
        match components[2] {
            CONSENSUS_STATE_PREFIX => {}
            _ => return None,
        }

        let epoch_height: Vec<&str> = components[3].split('-').collect();

        if epoch_height.len() != 2 {
            return None;
        }

        let revision_number = epoch_height[0];
        let revision_height = epoch_height[1];

        let revision_number = revision_number.parse::<u64>().ok()?;

        let revision_height = revision_height.parse::<u64>().ok()?;

        match components.len() {
            4 => Some(
                ClientConsensusStatePath {
                    client_id,
                    revision_number,
                    revision_height,
                }
                .into(),
            ),
            5 => match components[4] {
                PROCESSED_TIME => Some(
                    ClientUpdateTimePath {
                        client_id,
                        revision_number,
                        revision_height,
                    }
                    .into(),
                ),
                PROCESSED_HEIGHT => Some(
                    ClientUpdateHeightPath {
                        client_id,
                        revision_number,
                        revision_height,
                    }
                    .into(),
                ),
                _ => None,
            },
            _ => None,
        }
    } else {
        None
    }
}

fn parse_connections(components: &[&str]) -> Option<Path> {
    if components.len() != 2 {
        return None;
    }

    let first = *components.first()?;

    if first != CONNECTION_PREFIX {
        return None;
    }

    let connection_id = *components.last()?;

    let connection_id = ConnectionId::from_str(connection_id).ok()?;

    Some(ConnectionPath(connection_id).into())
}

fn parse_ports(components: &[&str]) -> Option<Path> {
    if components.len() != 2 {
        return None;
    }

    let first = *components.first()?;

    if first != PORT_PREFIX {
        return None;
    }

    let port_id = *components.last()?;

    let port_id = PortId::from_str(port_id).ok()?;

    Some(PortPath(port_id).into())
}

fn parse_channels(components: &[&str]) -> Option<SubPath> {
    if components.len() != 2 {
        return None;
    }

    let first = *components.first()?;

    if first != CHANNEL_PREFIX {
        return None;
    }

    let channel_id = *components.last()?;

    let channel_id = ChannelId::from_str(channel_id).ok()?;

    Some(SubPath::Channels(channel_id))
}

fn parse_sequences(components: &[&str]) -> Option<SubPath> {
    if components.len() != 2 {
        return None;
    }

    let first = *components.first()?;

    if first != SEQUENCE_PREFIX {
        return None;
    }

    let sequence_number = *components.last()?;

    match Sequence::from_str(sequence_number) {
        Ok(seq) => Some(SubPath::Sequences(seq)),
        Err(_) => None,
    }
}

fn parse_channel_ends(components: &[&str]) -> Option<Path> {
    if components.len() != 5 {
        return None;
    }

    let first = *components.first()?;

    if first != CHANNEL_END_PREFIX {
        return None;
    }

    let port = parse_ports(&components[1..=2]);
    let channel = parse_channels(&components[3..=4]);

    let Some(Path::Ports(PortPath(port_id))) = port else {
        return None;
    };

    let Some(SubPath::Channels(channel_id)) = channel else {
        return None;
    };

    Some(ChannelEndPath(port_id, channel_id).into())
}

fn parse_seqs(components: &[&str]) -> Option<Path> {
    if components.len() != 5 {
        return None;
    }

    let first = *components.first()?;

    let port = parse_ports(&components[1..=2]);
    let channel = parse_channels(&components[3..=4]);

    let Some(Path::Ports(PortPath(port_id))) = port else {
        return None;
    };

    let Some(SubPath::Channels(channel_id)) = channel else {
        return None;
    };

    match first {
        NEXT_SEQ_SEND_PREFIX => Some(SeqSendPath(port_id, channel_id).into()),
        NEXT_SEQ_RECV_PREFIX => Some(SeqRecvPath(port_id, channel_id).into()),
        NEXT_SEQ_ACK_PREFIX => Some(SeqAckPath(port_id, channel_id).into()),
        _ => None,
    }
}

fn parse_commitments(components: &[&str]) -> Option<Path> {
    if components.len() != 7 {
        return None;
    }

    let first = *components.first()?;

    if first != PACKET_COMMITMENT_PREFIX {
        return None;
    }

    let port = parse_ports(&components[1..=2]);
    let channel = parse_channels(&components[3..=4]);
    let sequence = parse_sequences(&components[5..]);

    let Some(Path::Ports(PortPath(port_id))) = port else {
        return None;
    };

    let Some(SubPath::Channels(channel_id)) = channel else {
        return None;
    };

    let Some(SubPath::Sequences(sequence)) = sequence else {
        return None;
    };

    Some(
        CommitmentPath {
            port_id,
            channel_id,
            sequence,
        }
        .into(),
    )
}

fn parse_acks(components: &[&str]) -> Option<Path> {
    if components.len() != 7 {
        return None;
    }

    let first = *components.first()?;

    if first != PACKET_ACK_PREFIX {
        return None;
    }

    let port = parse_ports(&components[1..=2]);
    let channel = parse_channels(&components[3..=4]);
    let sequence = parse_sequences(&components[5..]);

    let Some(Path::Ports(PortPath(port_id))) = port else {
        return None;
    };

    let Some(SubPath::Channels(channel_id)) = channel else {
        return None;
    };

    let Some(SubPath::Sequences(sequence)) = sequence else {
        return None;
    };

    Some(
        AckPath {
            port_id,
            channel_id,
            sequence,
        }
        .into(),
    )
}

fn parse_receipts(components: &[&str]) -> Option<Path> {
    if components.len() != 7 {
        return None;
    }

    let first = *components.first()?;

    if first != PACKET_RECEIPT_PREFIX {
        return None;
    }

    let port = parse_ports(&components[1..=2]);
    let channel = parse_channels(&components[3..=4]);
    let sequence = parse_sequences(&components[5..]);

    let Some(Path::Ports(PortPath(port_id))) = port else {
        return None;
    };

    let Some(SubPath::Channels(channel_id)) = channel else {
        return None;
    };

    let Some(SubPath::Sequences(sequence)) = sequence else {
        return None;
    };

    Some(
        ReceiptPath {
            port_id,
            channel_id,
            sequence,
        }
        .into(),
    )
}

fn parse_upgrades(components: &[&str]) -> Option<Path> {
    if components.len() != 3 {
        return None;
    }

    let first = *components.first()?;

    if first != UPGRADED_IBC_STATE {
        return None;
    }

    let last = *components.last()?;

    let height = components[1].parse::<u64>().ok()?;

    match last {
        UPGRADED_CLIENT_STATE => Some(UpgradeClientPath::UpgradedClientState(height).into()),
        UPGRADED_CLIENT_CONSENSUS_STATE => {
            Some(UpgradeClientPath::UpgradedClientConsensusState(height).into())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const DEFAULT_CLIENT_ID_STR: &str = "07-tendermint-0";
    impl ClientId {
        pub fn new_dummy() -> Self {
            ClientId::from_str(DEFAULT_CLIENT_ID_STR)
                .expect("should not fail since we use a valid client id")
        }
    }
    #[rstest::rstest]
    #[case(NEXT_CLIENT_SEQUENCE, Path::NextClientSequence(NextClientSequencePath))]
    #[case(
        NEXT_CONNECTION_SEQUENCE,
        Path::NextConnectionSequence(NextConnectionSequencePath)
    )]
    #[case(
        NEXT_CHANNEL_SEQUENCE,
        Path::NextChannelSequence(NextChannelSequencePath)
    )]
    #[case(
        "clients/07-tendermint-0/clientState",
        Path::ClientState(ClientStatePath(ClientId::new_dummy()))
    )]
    #[case(
        "clients/07-tendermint-0/consensusStates/15-31",
        Path::ClientConsensusState(ClientConsensusStatePath {
            client_id: ClientId::new_dummy(),
            revision_number: 15,
            revision_height: 31,
        })
    )]
    #[case(
        "clients/07-tendermint-0/consensusStates/15-31/processedTime",
        Path::ClientUpdateTime(ClientUpdateTimePath {
            client_id: ClientId::new_dummy(),
            revision_number: 15,
            revision_height: 31,
        })
    )]
    #[case(
        "clients/07-tendermint-0/consensusStates/15-31/processedHeight",
        Path::ClientUpdateHeight(ClientUpdateHeightPath {
            client_id: ClientId::new_dummy(),
            revision_number: 15,
            revision_height: 31,
        })
    )]
    #[case(
        "clients/07-tendermint-0/connections",
        Path::ClientConnection(ClientConnectionPath(ClientId::new_dummy()))
    )]
    #[case(
        "connections/connection-0",
        Path::Connection(ConnectionPath(ConnectionId::zero()))
    )]
    #[case("ports/transfer", Path::Ports(PortPath(PortId::transfer())))]
    #[case(
        "channelEnds/ports/transfer/channels/channel-0",
        Path::ChannelEnd(ChannelEndPath(PortId::transfer(), ChannelId::zero()))
    )]
    #[case(
        "nextSequenceSend/ports/transfer/channels/channel-0",
        Path::SeqSend(SeqSendPath(PortId::transfer(), ChannelId::zero()))
    )]
    #[case(
        "nextSequenceRecv/ports/transfer/channels/channel-0",
        Path::SeqRecv(SeqRecvPath(PortId::transfer(), ChannelId::zero()))
    )]
    #[case(
        "nextSequenceAck/ports/transfer/channels/channel-0",
        Path::SeqAck(SeqAckPath(PortId::transfer(), ChannelId::zero()))
    )]
    #[case(
        "commitments/ports/transfer/channels/channel-0/sequences/0",
        Path::Commitment(CommitmentPath {
            port_id: PortId::transfer(),
            channel_id: ChannelId::zero(),
            sequence: Sequence::from(0),
        })
    )]
    #[case(
        "acks/ports/transfer/channels/channel-0/sequences/0",
        Path::Ack(AckPath {
            port_id: PortId::transfer(),
            channel_id: ChannelId::zero(),
            sequence: Sequence::from(0),
        })
    )]
    #[case(
        "receipts/ports/transfer/channels/channel-0/sequences/0",
        Path::Receipt(ReceiptPath {
            port_id: PortId::transfer(),
            channel_id: ChannelId::zero(),
            sequence: Sequence::from(0),
        })
    )]
    #[case(
        "upgradedIBCState/0/upgradedClient",
        Path::UpgradeClient(UpgradeClientPath::UpgradedClientState(0))
    )]
    #[case(
        "upgradedIBCState/0/upgradedConsState",
        Path::UpgradeClient(UpgradeClientPath::UpgradedClientConsensusState(0))
    )]
    fn test_successful_parsing(#[case] path_str: &str, #[case] path: Path) {
        // can be parsed into Path
        assert_eq!(Path::from_str(path_str).expect("no error"), path);
        // can be converted back to string
        assert_eq!(path_str, path.to_string());
    }

    #[rstest::rstest]
    #[case("clients/clientType")]
    #[case("channels/channel-0")]
    #[case("sequences/0")]
    fn test_failure_parsing(#[case] path_str: &str) {
        // cannot be parsed into Path
        assert!(Path::from_str(path_str).is_err());
    }

    #[test]
    fn test_parse_client_paths_fn() {
        let path = "clients/07-tendermint-0/clientState";
        let components: Vec<&str> = path.split('/').collect();

        assert_eq!(
            parse_client_paths(&components),
            Some(Path::ClientState(ClientStatePath(ClientId::new_dummy())))
        );

        let path = "clients/07-tendermint-0/consensusStates/15-31";
        let components: Vec<&str> = path.split('/').collect();

        assert_eq!(
            parse_client_paths(&components),
            Some(Path::ClientConsensusState(ClientConsensusStatePath {
                client_id: ClientId::new_dummy(),
                revision_number: 15,
                revision_height: 31,
            }))
        );
    }

    #[test]
    fn test_parse_client_update_paths_fn() {
        let path = "clients/07-tendermint-0/consensusStates/15-31/processedTime";
        let components: Vec<&str> = path.split('/').collect();

        assert_eq!(
            parse_client_paths(&components),
            Some(Path::ClientUpdateTime(ClientUpdateTimePath {
                client_id: ClientId::new_dummy(),
                revision_number: 15,
                revision_height: 31,
            }))
        );

        let path = "clients/07-tendermint-0/consensusStates/15-31/processedHeight";
        let components: Vec<&str> = path.split('/').collect();

        assert_eq!(
            parse_client_paths(&components),
            Some(Path::ClientUpdateHeight(ClientUpdateHeightPath {
                client_id: ClientId::new_dummy(),
                revision_number: 15,
                revision_height: 31,
            }))
        );
    }

    #[test]
    fn test_parse_connections_fn() {
        let path = "connections/connection-0";
        let components: Vec<&str> = path.split('/').collect();

        assert_eq!(
            parse_connections(&components),
            Some(Path::Connection(ConnectionPath(ConnectionId::zero()))),
        );
    }

    #[test]
    fn test_parse_ports_fn() {
        let path = "ports/transfer";
        let components: Vec<&str> = path.split('/').collect();

        assert_eq!(
            parse_ports(&components),
            Some(Path::Ports(PortPath(PortId::transfer()))),
        );
    }

    #[test]
    fn test_parse_channels_fn() {
        let path = "channels/channel-0";
        let components: Vec<&str> = path.split('/').collect();

        assert_eq!(
            parse_channels(&components),
            Some(SubPath::Channels(ChannelId::zero())),
        );
    }

    #[test]
    fn test_parse_sequences_fn() {
        let path = "sequences/0";
        let components: Vec<&str> = path.split('/').collect();

        assert_eq!(
            parse_sequences(&components),
            Some(SubPath::Sequences(Sequence::from(0)))
        );
    }

    #[test]
    fn test_parse_channel_ends_fn() {
        let path = "channelEnds/ports/transfer/channels/channel-0";
        let components: Vec<&str> = path.split('/').collect();

        assert_eq!(
            parse_channel_ends(&components),
            Some(Path::ChannelEnd(ChannelEndPath(
                PortId::transfer(),
                ChannelId::zero()
            ))),
        );
    }

    #[test]
    fn test_parse_seqs_fn() {
        let path = "nextSequenceSend/ports/transfer/channels/channel-0";
        let components: Vec<&str> = path.split('/').collect();

        assert_eq!(
            parse_seqs(&components),
            Some(Path::SeqSend(SeqSendPath(
                PortId::transfer(),
                ChannelId::zero()
            ))),
        );

        let path = "nextSequenceRecv/ports/transfer/channels/channel-0";
        let components: Vec<&str> = path.split('/').collect();

        assert_eq!(
            parse_seqs(&components),
            Some(Path::SeqRecv(SeqRecvPath(
                PortId::transfer(),
                ChannelId::zero()
            ))),
        );

        let path = "nextSequenceAck/ports/transfer/channels/channel-0";
        let components: Vec<&str> = path.split('/').collect();

        assert_eq!(
            parse_seqs(&components),
            Some(Path::SeqAck(SeqAckPath(
                PortId::transfer(),
                ChannelId::zero()
            ))),
        );
    }

    #[test]
    fn test_parse_commitments_fn() {
        let path = "commitments/ports/transfer/channels/channel-0/sequences/0";
        let components: Vec<&str> = path.split('/').collect();

        assert_eq!(
            parse_commitments(&components),
            Some(Path::Commitment(CommitmentPath {
                port_id: PortId::transfer(),
                channel_id: ChannelId::zero(),
                sequence: Sequence::from(0),
            })),
        );
    }

    #[test]
    fn test_parse_acks_fn() {
        let path = "acks/ports/transfer/channels/channel-0/sequences/0";
        let components: Vec<&str> = path.split('/').collect();

        assert_eq!(
            parse_acks(&components),
            Some(Path::Ack(AckPath {
                port_id: PortId::transfer(),
                channel_id: ChannelId::zero(),
                sequence: Sequence::from(0),
            })),
        );
    }

    #[test]
    fn test_parse_receipts_fn() {
        let path = "receipts/ports/transfer/channels/channel-0/sequences/0";
        let components: Vec<&str> = path.split('/').collect();

        assert_eq!(
            parse_receipts(&components),
            Some(Path::Receipt(ReceiptPath {
                port_id: PortId::transfer(),
                channel_id: ChannelId::zero(),
                sequence: Sequence::from(0),
            })),
        );
    }

    #[test]
    fn test_parse_upgrades_fn() {
        let path = "upgradedIBCState/0/upgradedClient";
        let components: Vec<&str> = path.split('/').collect();

        assert_eq!(
            parse_upgrades(&components),
            Some(Path::UpgradeClient(UpgradeClientPath::UpgradedClientState(
                0
            ))),
        );

        let path = "upgradedIBCState/0/upgradedConsState";
        let components: Vec<&str> = path.split('/').collect();

        assert_eq!(
            parse_upgrades(&components),
            Some(Path::UpgradeClient(
                UpgradeClientPath::UpgradedClientConsensusState(0)
            )),
        )
    }
}
