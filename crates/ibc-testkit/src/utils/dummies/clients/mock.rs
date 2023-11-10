use ibc::Height;

use crate::testapp::ibc::clients::mock::header::MockHeader;

pub fn dummy_new_mock_header(revision_height: u64) -> MockHeader {
    MockHeader::new(Height::new(0, revision_height).expect("Never fails"))
}
