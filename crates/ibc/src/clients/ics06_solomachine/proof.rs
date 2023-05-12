use alloc::string::String;
use alloc::vec::Vec;
use cosmrs::crypto::PublicKey;
// use cosmrs::tx::SignatureBytes;

pub fn verify_signature(publik_key: PublicKey, sign_bytes: Vec<u8>) -> Result<(), String> {
    todo!()
}
