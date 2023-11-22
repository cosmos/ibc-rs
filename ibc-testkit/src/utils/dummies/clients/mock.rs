use ibc::core::client::types::Height;

use crate::testapp::ibc::clients::mock::header::MockHeader;
/// Returns a dummy `MockHeader` with the given revision height.
pub fn dummy_new_mock_header(revision_height: u64) -> MockHeader {
    MockHeader::new(Height::new(0, revision_height).expect("Never fails"))
}
