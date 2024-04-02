use ibc::core::client::context::ClientValidationContext;
use ibc::core::client::types::msgs::{ClientMsg, MsgCreateClient, MsgUpdateClient};
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

    // Create the subject client
    let msg = MsgCreateClient::new(
        MockClientState::new(MockHeader::new(height)).into(),
        MockConsensusState::new(MockHeader::new(height)).into(),
        signer.clone(),
    );

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg));

    let client_type = mock_client_type();
    let subject_client_id = client_type.build_client_id(ctx.client_counter().unwrap());

    validate(&ctx, &router, msg_envelope.clone()).expect("create subject client validation");
    execute(&mut ctx, &mut router, msg_envelope).expect("create subject client execution");

    // Create the substitute client
    let height = height.increment();

    let msg = MsgCreateClient::new(
        MockClientState::new(MockHeader::new(height)).into(),
        MockConsensusState::new(MockHeader::new(height)).into(),
        signer.clone(),
    );

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg));

    let substitute_client_id = client_type.build_client_id(ctx.client_counter().unwrap());

    validate(&ctx, &router, msg_envelope.clone()).expect("create substitute client validation");
    execute(&mut ctx, &mut router, msg_envelope).expect("create substitute client execution");

    // Perform a client updates on the subject and substitute clients
    let update_subject_msg = MsgUpdateClient {
        client_id: subject_client_id.clone(),
        client_message: MockHeader::new(height).with_timestamp(timestamp).into(),
        signer: signer.clone(),
    };

    let update_subject_msg_envelope = MsgEnvelope::from(ClientMsg::from(update_subject_msg));

    validate(&ctx, &router, update_subject_msg_envelope.clone())
        .expect("update subject client validation");
    execute(&mut ctx, &mut router, update_subject_msg_envelope)
        .expect("update subject client execution");

    let update_substitute_msg = MsgUpdateClient {
        client_id: substitute_client_id.clone(),
        client_message: MockHeader::new(height).with_timestamp(timestamp).into(),
        signer: signer.clone(),
    };

    let update_substitute_msg_envelope = MsgEnvelope::from(ClientMsg::from(update_substitute_msg));

    validate(&ctx, &router, update_substitute_msg_envelope.clone())
        .expect("update substitute client validation");
    execute(&mut ctx, &mut router, update_substitute_msg_envelope)
        .expect("update substitute client execution");

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

    if let Err(ref e) = res {
        eprintln!("validation happy path: {e}");
    }

    assert!(res.is_ok(), "validation happy path");

    let res = execute(&mut ctx, &mut router, msg_envelope);

    if let Err(ref e) = res {
        eprintln!("execution happy path: {e}");
    }

    assert!(res.is_ok(), "execution happy path");

    assert_eq!(
        ctx.client_state(&msg.subject_client_id).unwrap(),
        ctx.client_state(&msg.substitute_client_id).unwrap(),
    )
}
