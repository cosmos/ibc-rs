use crate::core::ics04_channel::msgs::acknowledgement::Acknowledgement;
use crate::core::ics04_channel::timeout::TimeoutHeight;
use crate::prelude::*;
use crate::timestamp::Timestamp;

/// Packet commitment
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PacketCommitment(Vec<u8>);

impl PacketCommitment {
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl AsRef<[u8]> for PacketCommitment {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for PacketCommitment {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

/// Acknowledgement commitment to be stored
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcknowledgementCommitment(Vec<u8>);

impl AcknowledgementCommitment {
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl AsRef<[u8]> for AcknowledgementCommitment {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for AcknowledgementCommitment {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

/// Compute the commitment for a packet.
///
/// Note that the absence of `timeout_height` is treated as
/// `{revision_number: 0, revision_height: 0}` to be consistent with ibc-go,
/// where this value is used to mean "no timeout height":
/// <https://github.com/cosmos/ibc-go/blob/04791984b3d6c83f704c4f058e6ca0038d155d91/modules/core/04-channel/keeper/packet.go#L206>
pub(crate) fn compute_packet_commitment(
    packet_data: &[u8],
    timeout_height: &TimeoutHeight,
    timeout_timestamp: &Timestamp,
) -> PacketCommitment {
    let mut hash_input = timeout_timestamp.nanoseconds().to_be_bytes().to_vec();

    let revision_number = timeout_height.commitment_revision_number().to_be_bytes();
    hash_input.append(&mut revision_number.to_vec());

    let revision_height = timeout_height.commitment_revision_height().to_be_bytes();
    hash_input.append(&mut revision_height.to_vec());

    let packet_data_hash = hash(packet_data);
    hash_input.append(&mut packet_data_hash.to_vec());

    hash(&hash_input).into()
}

/// Compute the commitment for an acknowledgement.
pub(crate) fn compute_ack_commitment(ack: &Acknowledgement) -> AcknowledgementCommitment {
    hash(ack.as_ref()).into()
}

/// Helper function to hash a byte slice using SHA256.
///
/// Note that computing commitments with anything other than SHA256 will
/// break the Merkle proofs of the IBC provable store.
fn hash(data: impl AsRef<[u8]>) -> Vec<u8> {
    use sha2::Digest;

    sha2::Sha256::digest(&data).to_vec()
}
