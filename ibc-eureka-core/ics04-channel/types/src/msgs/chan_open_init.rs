use ibc_eureka_core_host_types::error::DecodingError;
use ibc_eureka_core_host_types::identifiers::PortId;
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::ibc::core::channel::v1::MsgChannelOpenInit as RawMsgChannelOpenInit;
use ibc_proto::Protobuf;

use crate::channel::{ChannelEnd, Counterparty, Order, State};
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
    pub port_id_on_b: PortId,
    pub ordering: Order,
    pub signer: Signer,
    /// Allow a relayer to specify a particular version by providing a non-empty version string
    pub version_proposal: Version,
}

impl Protobuf<RawMsgChannelOpenInit> for MsgChannelOpenInit {}

impl TryFrom<RawMsgChannelOpenInit> for MsgChannelOpenInit {
    type Error = DecodingError;

    fn try_from(raw_msg: RawMsgChannelOpenInit) -> Result<Self, Self::Error> {
        let chan_end_on_a: ChannelEnd = raw_msg
            .channel
            .ok_or(DecodingError::missing_raw_data("channel end"))?
            .try_into()?;

        chan_end_on_a
            .verify_state_matches(&State::Init)
            .map_err(|_| {
                DecodingError::invalid_raw_data(format!(
                    "expected channel end to be in `Init` state but it is in `{}` instead",
                    chan_end_on_a.state
                ))
            })?;

        if let Some(cid) = chan_end_on_a.counterparty().channel_id() {
            return Err(DecodingError::invalid_raw_data(format!(
                "expected counterparty channel ID to be empty, actual `{cid}`",
            )));
        }

        Ok(MsgChannelOpenInit {
            port_id_on_a: raw_msg.port_id.parse()?,
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
            domain_msg.version_proposal,
        );
        RawMsgChannelOpenInit {
            port_id: domain_msg.port_id_on_a.to_string(),
            channel: Some(chan_end_on_a.into()),
            signer: domain_msg.signer.to_string(),
        }
    }
}
