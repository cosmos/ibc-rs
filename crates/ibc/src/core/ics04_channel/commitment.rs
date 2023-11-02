//! Types and utilities related to packet commitments.

use super::acknowledgement::Acknowledgement;
use crate::core::ics04_channel::timeout::TimeoutHeight;
use crate::core::timestamp::Timestamp;
use crate::prelude::*;

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
    let mut hash_input = [0; 8 * 3 + 32];

    hash_input[..8].copy_from_slice(&timeout_timestamp.nanoseconds().to_be_bytes());
    hash_input[8..16].copy_from_slice(&timeout_height.commitment_revision_number().to_be_bytes());
    hash_input[16..24].copy_from_slice(&timeout_height.commitment_revision_height().to_be_bytes());
    hash_input[24..].copy_from_slice(&hash(packet_data));

    hash(&hash_input).to_vec().into()
}

/// Compute the commitment for an acknowledgement.
pub(crate) fn compute_ack_commitment(ack: &Acknowledgement) -> AcknowledgementCommitment {
    hash(ack.as_ref()).to_vec().into()
}

/// Helper function to hash a byte slice using SHA256.
///
/// Note that computing commitments with anything other than SHA256 will
/// break the Merkle proofs of the IBC provable store.
fn hash(data: &[u8]) -> [u8; 32] {
    use sha2::Digest;

    sha2::Sha256::digest(data).into()
}

#[test]
fn test_compute_commitment() {
    // PacketCommitment
    let want: [u8; 32] = [
        169, 40, 181, 31, 98, 189, 84, 0, 145, 236, 69, 31, 78, 243, 69, 121, 79, 5, 158, 101, 145,
        8, 22, 134, 97, 38, 220, 54, 79, 132, 204, 21,
    ];
    let got = compute_packet_commitment(
        "packet data".as_bytes(),
        &TimeoutHeight::At(crate::Height::new(42, 24).unwrap()),
        &Timestamp::from_nanoseconds(0x42).unwrap(),
    );
    assert_eq!(&want[..], got.as_ref());

    // AcknowledgementCommitment
    let want: [u8; 32] = [
        5, 78, 222, 193, 208, 33, 31, 98, 79, 237, 12, 188, 169, 212, 249, 64, 11, 14, 73, 28, 67,
        116, 42, 242, 197, 176, 171, 235, 240, 201, 144, 216,
    ];
    let ack = Acknowledgement::try_from(vec![0u8, 1, 2, 3]).unwrap();
    let got = compute_ack_commitment(&ack);
    assert_eq!(&want[..], got.as_ref())
}
