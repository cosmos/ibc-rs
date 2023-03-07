use sha2::Digest;

/// Helper function to hash a byte slice using SHA256.
///
/// Note that computing commitments & traces with anything other than SHA256
/// will break the Merkle proofs of the IBC provable store.
pub fn hash(data: impl AsRef<[u8]>) -> [u8; 32] {
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}
