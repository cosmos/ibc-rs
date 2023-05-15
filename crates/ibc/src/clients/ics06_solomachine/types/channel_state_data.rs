use crate::clients::ics06_solomachine::error::Error;
use crate::core::ics04_channel::channel::ChannelEnd;
use crate::prelude::*;
use ibc_proto::ibc::lightclients::solomachine::v2::ChannelStateData as RawChannelStateData;
use ibc_proto::protobuf::Protobuf;

/// ChannelStateData returns the SignBytes data for channel state
/// verification.
#[derive(Clone, PartialEq)]
pub struct ChannelStateData {
    pub path: Vec<u8>,
    pub channel: ChannelEnd,
}
impl Protobuf<RawChannelStateData> for ChannelStateData {}

impl TryFrom<RawChannelStateData> for ChannelStateData {
    type Error = Error;

    fn try_from(raw: RawChannelStateData) -> Result<Self, Self::Error> {
        Ok(Self {
            path: raw.path,
            channel: raw
                .channel
                .ok_or(Error::ChannelEndIsEmpty)?
                .try_into()
                .map_err(Error::ChannelError)?,
        })
    }
}

impl From<ChannelStateData> for RawChannelStateData {
    fn from(value: ChannelStateData) -> Self {
        Self {
            path: value.path,
            channel: Some(value.channel.into()),
        }
    }
}
