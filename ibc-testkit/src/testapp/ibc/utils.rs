use ibc::core::client::types::Height;

pub fn blocks_since(a: Height, b: Height) -> Option<u64> {
    (a.revision_number() == b.revision_number() && a.revision_height() >= b.revision_height())
        .then(|| a.revision_height() - b.revision_height())
}
