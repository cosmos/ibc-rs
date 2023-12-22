use ibc_core_client_types::Height;
use ibc_core_commitment_types::commitment::CommitmentProofBytes;
use ibc_core_host_types::identifiers::{ChannelId, ConnectionId, PortId};
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::ibc::core::channel::v1::MsgChannelOpenTry as RawMsgChannelOpenTry;
use ibc_proto::Protobuf;

use crate::channel::{verify_connection_hops_length, ChannelEnd, Counterparty, Order, State};
use crate::error::ChannelError;
use crate::Version;

pub const CHAN_OPEN_TRY_TYPE_URL: &str = "/ibc.core.channel.v1.MsgChannelOpenTry";

///
/// Message definition for the second step in the channel open handshake (`ChanOpenTry` datagram).
/// Per our convention, this message is sent to chain B.
///
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgChannelOpenTry {
    pub port_id_on_b: PortId,
    pub connection_hops_on_b: Vec<ConnectionId>,
    pub port_id_on_a: PortId,
    pub chan_id_on_a: ChannelId,
    pub version_supported_on_a: Version,
    pub proof_chan_end_on_a: CommitmentProofBytes,
    pub proof_height_on_a: Height,
    pub ordering: Order,
    pub signer: Signer,

    #[deprecated(since = "0.22.0")]
    /// Only kept here for proper conversion to/from the raw type
    pub version_proposal: Version,
}

impl MsgChannelOpenTry {
    /// Checks if the `connection_hops` has a length of `expected`.
    ///
    /// Note: Current IBC version only supports one connection hop.
    pub fn verify_connection_hops_length(&self) -> Result<(), ChannelError> {
        verify_connection_hops_length(&self.connection_hops_on_b, 1)
    }
}

impl Protobuf<RawMsgChannelOpenTry> for MsgChannelOpenTry {}

impl TryFrom<RawMsgChannelOpenTry> for MsgChannelOpenTry {
    type Error = ChannelError;

    fn try_from(raw_msg: RawMsgChannelOpenTry) -> Result<Self, Self::Error> {
        let chan_end_on_b: ChannelEnd = raw_msg
            .channel
            .ok_or(ChannelError::MissingChannel)?
            .try_into()?;

        chan_end_on_b.verify_state_matches(&State::TryOpen)?;

        #[allow(deprecated)]
        if !raw_msg.previous_channel_id.is_empty() {
            return Err(ChannelError::InvalidChannelId {
                expected: "previous channel id must be empty. It has been deprecated as crossing hellos are no longer supported".to_string(),
                actual: raw_msg.previous_channel_id,
            });
        }

        #[allow(deprecated)]
        let msg = MsgChannelOpenTry {
            port_id_on_b: raw_msg.port_id.parse()?,
            ordering: chan_end_on_b.ordering,
            connection_hops_on_b: chan_end_on_b.connection_hops,
            port_id_on_a: chan_end_on_b.remote.port_id,
            chan_id_on_a: chan_end_on_b
                .remote
                .channel_id
                .ok_or(ChannelError::MissingCounterparty)?,
            version_supported_on_a: raw_msg.counterparty_version.into(),
            proof_chan_end_on_a: raw_msg
                .proof_init
                .try_into()
                .map_err(|_| ChannelError::InvalidProof)?,
            proof_height_on_a: raw_msg
                .proof_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(ChannelError::MissingHeight)?,
            signer: raw_msg.signer.into(),
            version_proposal: chan_end_on_b.version,
        };

        Ok(msg)
    }
}

impl From<MsgChannelOpenTry> for RawMsgChannelOpenTry {
    fn from(domain_msg: MsgChannelOpenTry) -> Self {
        let chan_end_on_b = ChannelEnd::new_without_validation(
            State::TryOpen,
            domain_msg.ordering,
            Counterparty::new(domain_msg.port_id_on_a, Some(domain_msg.chan_id_on_a)),
            domain_msg.connection_hops_on_b,
            Version::empty(), // Excessive field to satisfy the type conversion
        );
        #[allow(deprecated)]
        RawMsgChannelOpenTry {
            port_id: domain_msg.port_id_on_b.to_string(),
            previous_channel_id: "".to_string(), // Excessive field to satisfy the type conversion",
            channel: Some(chan_end_on_b.into()),
            counterparty_version: domain_msg.version_supported_on_a.to_string(),
            proof_init: domain_msg.proof_chan_end_on_a.clone().into(),
            proof_height: Some(domain_msg.proof_height_on_a.into()),
            signer: domain_msg.signer.to_string(),
        }
    }
}
