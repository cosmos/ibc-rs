use ibc::core::client::types::msgs::ClientMsg;
use ibc::core::client::types::{msgs::MsgRecoverClient, Height};
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::types::identifiers::ClientId;
use ibc_testkit::{
    fixtures::core::signer::dummy_account_id,
    testapp::ibc::core::types::{MockClientConfig, MockContext},
};

use rstest::*;

struct Fixture {
    ctx: MockContext,
}

#[fixture]
fn fixture() -> Fixture {
    let client_id = ClientId::new("07-tendermint", 0).expect("no error");

    let ctx = MockContext::default().with_client_config(
        MockClientConfig::builder()
            .client_id(client_id.clone())
            .latest_height(Height::new(0, 42).unwrap())
            .build(),
    );

    Fixture { ctx }
}

#[rstest]
fn test_recover_client_ok(fixture: Fixture) {
    let Fixture { ctx } = fixture;

    let subject_client_id = ClientId::new("07-tendermint", 0).expect("no error");
    let substitute_client_id = ClientId::new("07-tendermint", 1).expect("no error");
    let signer = dummy_account_id();

    let msg = MsgRecoverClient {
        subject_client_id,
        substitute_client_id,
        signer,
    };

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg.clone()));
}
