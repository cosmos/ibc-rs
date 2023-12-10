use ibc_core_host_types::identifiers::{ConnectionId, PortId};
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::ibc::core::channel::v1::MsgChannelOpenInit as RawMsgChannelOpenInit;
use ibc_proto::Protobuf;

use crate::channel::{verify_connection_hops_length, ChannelEnd, Counterparty, Order, State};
use crate::error::ChannelError;
use crate::Version;

pub const CHAN_OPEN_INIT_TYPE_URL: &str = "/ibc.core.channel.v1.MsgChannelOpenInit";

///
/// Message definition for the first step in the channel open handshake (`ChanOpenInit` datagram).
/// Per our convention, this message is sent to chain A.
///
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgChannelOpenInit {
    pub port_id_on_a: PortId,
    pub connection_hops_on_a: Vec<ConnectionId>,
    pub port_id_on_b: PortId,
    pub ordering: Order,
    pub signer: Signer,
    /// Allow a relayer to specify a particular version by providing a non-empty version string
    pub version_proposal: Version,
}

impl MsgChannelOpenInit {
    /// Checks if the `connection_hops` has a length of `expected`.
    ///
    /// Note: Current IBC version only supports one connection hop.
    pub fn verify_connection_hops_length(&self) -> Result<(), ChannelError> {
        verify_connection_hops_length(&self.connection_hops_on_a, 1)
    }
}

impl Protobuf<RawMsgChannelOpenInit> for MsgChannelOpenInit {}

impl TryFrom<RawMsgChannelOpenInit> for MsgChannelOpenInit {
    type Error = ChannelError;

    fn try_from(raw_msg: RawMsgChannelOpenInit) -> Result<Self, Self::Error> {
        let chan_end_on_a: ChannelEnd = raw_msg
            .channel
            .ok_or(ChannelError::MissingChannel)?
            .try_into()?;
        chan_end_on_a.verify_state_matches(&State::Init)?;
        chan_end_on_a.counterparty().verify_empty_channel_id()?;

        Ok(MsgChannelOpenInit {
            port_id_on_a: raw_msg.port_id.parse()?,
            connection_hops_on_a: chan_end_on_a.connection_hops,
            port_id_on_b: chan_end_on_a.remote.port_id,
            ordering: chan_end_on_a.ordering,
            signer: raw_msg.signer.into(),
            version_proposal: chan_end_on_a.version,
        })
    }
}

impl From<MsgChannelOpenInit> for RawMsgChannelOpenInit {
    fn from(domain_msg: MsgChannelOpenInit) -> Self {
        let chan_end_on_a = ChannelEnd::new_without_validation(
            State::Init,
            domain_msg.ordering,
            Counterparty::new(domain_msg.port_id_on_b, None),
            domain_msg.connection_hops_on_a,
            domain_msg.version_proposal,
        );
        RawMsgChannelOpenInit {
            port_id: domain_msg.port_id_on_a.to_string(),
            channel: Some(chan_end_on_a.into()),
            signer: domain_msg.signer.to_string(),
        }
    }
}
