use super::misbehaviour::signature_and_data::SignatureAndData;
use crate::clients::ics06_solomachine::error::Error;
use alloc::vec::Vec;
use cosmrs::crypto::PublicKey;

// Verify_signature verifies if the the provided public key generated the signature
// over the given data. Single and Multi signature public keys are supported.
// The signature data type must correspond to the public key type. An error is
// returned if signature verification fails or an invalid SignatureData type is
// provided.
pub fn verify_signature(
    _publik_key: PublicKey,
    _sign_bytes: Vec<u8>,
    _signature_and_data: SignatureAndData,
) -> Result<(), Error> {
    todo!()
}
