//! Types and utilities related to packet commitments.

use ibc_primitives::prelude::*;
use ibc_primitives::Timestamp;

use super::acknowledgement::Acknowledgement;
use crate::timeout::TimeoutHeight;

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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
pub fn compute_packet_commitment(
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
pub fn compute_ack_commitment(ack: &Acknowledgement) -> AcknowledgementCommitment {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_compute_packet_commitment() {
        let expected: [u8; 32] = [
            0xa9, 0x28, 0xb5, 0x1f, 0x62, 0xbd, 0x54, 0x00, 0x91, 0xec, 0x45, 0x1f, 0x4e, 0xf3,
            0x45, 0x79, 0x4f, 0x05, 0x9e, 0x65, 0x91, 0x08, 0x16, 0x86, 0x61, 0x26, 0xdc, 0x36,
            0x4f, 0x84, 0xcc, 0x15,
        ];
        let actual = compute_packet_commitment(
            b"packet data",
            &TimeoutHeight::At(ibc_core_client_types::Height::new(42, 24).unwrap()),
            &Timestamp::from_nanoseconds(0x42).unwrap(),
        );
        assert_eq!(&expected[..], actual.as_ref());
    }

    #[test]
    fn test_compute_ack_commitment() {
        let expected: [u8; 32] = [
            0x05, 0x4e, 0xde, 0xc1, 0xd0, 0x21, 0x1f, 0x62, 0x4f, 0xed, 0x0c, 0xbc, 0xa9, 0xd4,
            0xf9, 0x40, 0x0b, 0x0e, 0x49, 0x1c, 0x43, 0x74, 0x2a, 0xf2, 0xc5, 0xb0, 0xab, 0xeb,
            0xf0, 0xc9, 0x90, 0xd8,
        ];
        let ack = Acknowledgement::try_from(vec![0, 1, 2, 3]).unwrap();
        let actual = compute_ack_commitment(&ack);
        assert_eq!(&expected[..], actual.as_ref())
    }
}
