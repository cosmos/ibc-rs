use ibc::core::client::types::msgs::ClientMsg;
use ibc::core::client::types::{msgs::MsgRecoverClient, Height};
use ibc::core::entrypoint::{execute, validate};
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::types::identifiers::ClientId;
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::{
    fixtures::core::signer::dummy_account_id,
    testapp::ibc::core::types::{MockClientConfig, MockContext},
};

use rstest::*;

struct Fixture {
    ctx: MockContext,
    router: MockRouter,
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

    let router = MockRouter::new_with_transfer();

    Fixture { ctx, router }
}

#[rstest]
fn test_recover_client_ok(fixture: Fixture) {
    let Fixture {
        mut ctx,
        mut router,
    } = fixture;

    let subject_client_id = ClientId::new("07-tendermint", 0).expect("no error");
    let substitute_client_id = ClientId::new("07-tendermint", 1).expect("no error");
    let signer = dummy_account_id();

    let msg = MsgRecoverClient {
        subject_client_id,
        substitute_client_id,
        signer,
    };

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg.clone()));

    let res = validate(&ctx, &router, msg_envelope.clone());

    assert!(res.is_ok(), "validation happy path");

    let res = execute(&mut ctx, &mut router, msg_envelope);

    assert!(res.is_ok(), "execution happy path");

    // assert that the subject's client state is as expected
}
