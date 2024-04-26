use core::time::Duration;

use ibc::core::client::context::ClientValidationContext;
use ibc::core::client::handler::recover_client;
use ibc::core::client::types::msgs::{ClientMsg, MsgCreateClient, MsgRecoverClient};
use ibc::core::client::types::Height;
use ibc::core::entrypoint::{execute, validate};
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::types::identifiers::ClientId;
use ibc::core::host::types::path::ClientConsensusStatePath;
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
    subject_client_id: ClientId,
    substitute_client_id: ClientId,
    signer: Signer,
}

/// Initializes the testing fixture for validating client recovery logic.
///
/// Creates the subject and substitute clients via sending two `MsgCreateClient`
/// messages. The subject client is initialized in an expired state by sleeping
/// for the duration of its trusting period. The substitute client state is
/// initialized with a much longer trusting period and at a height higher than
/// the subject client state's in order to ensure that it remains in an active
/// state.
fn setup_client_recovery_fixture(
    subject_trusting_period: Duration,
    subject_height: Height,
    substitute_trusting_period: Duration,
    substitute_height: Height,
) -> Fixture {
    let mut ctx = MockContext::default();
    let mut router = MockRouter::new_with_transfer();
    let signer = dummy_account_id();

    let subject_timestamp = (Timestamp::now() - subject_trusting_period).unwrap();

    // Create the subject client state such that it will be in an expired state by initializing it with
    // a timestamp that is of duration `subject_trusting_period` in the past
    let subject_client_state =
        MockClientState::new(MockHeader::new(subject_height).with_timestamp(subject_timestamp))
            .with_trusting_period(subject_trusting_period);

    // Create the subject client
    let msg = MsgCreateClient::new(
        subject_client_state.into(),
        MockConsensusState::new(MockHeader::new(subject_height).with_current_timestamp()).into(),
        signer.clone(),
    );

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg));

    let client_type = mock_client_type();
    let subject_client_id = client_type.build_client_id(ctx.client_counter().unwrap());

    validate(&ctx, &router, msg_envelope.clone()).expect("create subject client validation");
    execute(&mut ctx, &mut router, msg_envelope).expect("create subject client execution");

    // Create the substitute client
    let substitute_client_state =
        MockClientState::new(MockHeader::new(substitute_height).with_current_timestamp())
            .with_trusting_period(substitute_trusting_period);

    let msg = MsgCreateClient::new(
        substitute_client_state.into(),
        MockConsensusState::new(MockHeader::new(substitute_height).with_current_timestamp()).into(),
        signer.clone(),
    );

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg));

    let substitute_client_id = client_type.build_client_id(ctx.client_counter().unwrap());

    validate(&ctx, &router, msg_envelope.clone()).expect("create substitute client validation");
    execute(&mut ctx, &mut router, msg_envelope).expect("create substitute client execution");

    Fixture {
        ctx,
        subject_client_id,
        substitute_client_id,
        signer,
    }
}

#[rstest]
fn test_recover_client_ok() {
    let subject_trusting_period = Duration::from_nanos(100);
    let substitute_trusting_period = Duration::from_secs(3);
    let subject_height = Height::new(0, 42).unwrap();
    let substitute_height = Height::new(0, 43).unwrap();

    let Fixture {
        mut ctx,
        subject_client_id,
        substitute_client_id,
        signer,
    } = setup_client_recovery_fixture(
        subject_trusting_period,
        subject_height,
        substitute_trusting_period,
        substitute_height,
    );

    let msg = MsgRecoverClient {
        subject_client_id,
        substitute_client_id,
        signer,
    };

    let res = recover_client::validate(&ctx, msg.clone());

    assert!(res.is_ok(), "client recovery validation happy path");

    let res = recover_client::execute(&mut ctx, msg.clone());

    assert!(res.is_ok(), "client recovery execution happy path");

    // client state is copied.
    assert_eq!(
        ctx.client_state(&msg.subject_client_id).unwrap(),
        ctx.client_state(&msg.substitute_client_id).unwrap(),
    );

    // latest consensus state is copied.
    assert_eq!(
        ctx.consensus_state(&ClientConsensusStatePath::new(
            msg.subject_client_id,
            substitute_height.revision_number(),
            substitute_height.revision_height(),
        ))
        .unwrap(),
        ctx.consensus_state(&ClientConsensusStatePath::new(
            msg.substitute_client_id,
            substitute_height.revision_number(),
            substitute_height.revision_height(),
        ))
        .unwrap(),
    );
}

#[rstest]
fn test_recover_client_with_expired_substitute() {
    let subject_trusting_period = Duration::from_nanos(100);
    let substitute_trusting_period = Duration::from_nanos(100);
    let subject_height = Height::new(0, 42).unwrap();
    let substitute_height = Height::new(0, 43).unwrap();

    let Fixture {
        ctx,
        subject_client_id,
        substitute_client_id,
        signer,
    } = setup_client_recovery_fixture(
        subject_trusting_period,
        subject_height,
        substitute_trusting_period,
        substitute_height,
    );

    let msg = MsgRecoverClient {
        subject_client_id,
        substitute_client_id,
        signer,
    };

    let res = recover_client::validate(&ctx, msg);

    assert!(res.is_err(), "expected client recovery validation to fail");
}

#[rstest]
fn test_recover_client_with_matching_heights() {
    let subject_trusting_period = Duration::from_nanos(100);
    let substitute_trusting_period = Duration::from_secs(3);
    let subject_height = Height::new(0, 42).unwrap();
    let substitute_height = Height::new(0, 42).unwrap();

    let Fixture {
        ctx,
        subject_client_id,
        substitute_client_id,
        signer,
    } = setup_client_recovery_fixture(
        subject_trusting_period,
        subject_height,
        substitute_trusting_period,
        substitute_height,
    );

    let msg = MsgRecoverClient {
        subject_client_id,
        substitute_client_id,
        signer,
    };

    let res = recover_client::validate(&ctx, msg);

    assert!(res.is_err(), "expected client recovery validation to fail");
}
