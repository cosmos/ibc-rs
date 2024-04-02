use core::time::Duration;
use std::thread::sleep;

use ibc::core::client::context::ClientValidationContext;
use ibc::core::client::types::msgs::{ClientMsg, MsgCreateClient};
use ibc::core::client::types::{msgs::MsgRecoverClient, Height};
use ibc::core::entrypoint::{execute, validate};
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::types::identifiers::ClientId;
use ibc::core::host::ValidationContext;
use ibc::core::primitives::{Signer, Timestamp};

use ibc_testkit::fixtures::core::signer::dummy_account_id;
use ibc_testkit::testapp::ibc::clients::mock::client_state::{
    client_type as mock_client_type, MockClientState,
};
use ibc_testkit::testapp::ibc::clients::mock::consensus_state::MockConsensusState;
use ibc_testkit::testapp::ibc::clients::mock::header::MockHeader;
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::MockContext;

use rstest::*;

struct Fixture {
    ctx: MockContext,
    router: MockRouter,
    subject_client_id: ClientId,
    substitute_client_id: ClientId,
    signer: Signer,
}

#[fixture]
fn fixture() -> Fixture {
    let mut ctx = MockContext::default();
    let mut router = MockRouter::new_with_transfer();
    let signer = dummy_account_id();
    let height = Height::new(0, 42).unwrap();
    let timestamp = Timestamp::now();

    let subject_client_state =
        MockClientState::new(MockHeader::new(height).with_timestamp(timestamp))
            .with_trusting_period(Duration::from_nanos(100));

    // Create the subject client
    let msg = MsgCreateClient::new(
        subject_client_state.into(),
        MockConsensusState::new(MockHeader::new(height).with_timestamp(timestamp)).into(),
        signer.clone(),
    );

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg));

    let client_type = mock_client_type();
    let subject_client_id = client_type.build_client_id(ctx.client_counter().unwrap());

    validate(&ctx, &router, msg_envelope.clone()).expect("create subject client validation");
    execute(&mut ctx, &mut router, msg_envelope).expect("create subject client execution");

    sleep(Duration::from_nanos(100));

    // Create the substitute client
    let height = height.increment();

    let substitute_client_state =
        MockClientState::new(MockHeader::new(height).with_timestamp(timestamp))
            .with_trusting_period(Duration::from_secs(3));

    let msg = MsgCreateClient::new(
        substitute_client_state.into(),
        MockConsensusState::new(MockHeader::new(height).with_timestamp(timestamp)).into(),
        signer.clone(),
    );

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg));

    let substitute_client_id = client_type.build_client_id(ctx.client_counter().unwrap());

    validate(&ctx, &router, msg_envelope.clone()).expect("create substitute client validation");
    execute(&mut ctx, &mut router, msg_envelope).expect("create substitute client execution");

    Fixture {
        ctx,
        router,
        subject_client_id,
        substitute_client_id,
        signer,
    }
}

#[rstest]
fn test_recover_client_ok(fixture: Fixture) {
    let Fixture {
        mut ctx,
        mut router,
        subject_client_id,
        substitute_client_id,
        signer,
    } = fixture;

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

    assert_eq!(
        ctx.client_state(&msg.subject_client_id).unwrap(),
        ctx.client_state(&msg.substitute_client_id).unwrap(),
    )
}
