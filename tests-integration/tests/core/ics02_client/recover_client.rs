use core::time::Duration;

use ibc::core::client::context::client_state::ClientStateValidation;
use ibc::core::client::context::ClientValidationContext;
use ibc::core::client::handler::recover_client;
use ibc::core::client::types::msgs::{ClientMsg, MsgCreateClient, MsgRecoverClient};
use ibc::core::client::types::{Height, Status as ClientStatus};
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::types::identifiers::ClientId;
use ibc::core::host::types::path::ClientConsensusStatePath;
use ibc::core::host::ValidationContext;
use ibc::core::primitives::Signer;
use ibc_primitives::Timestamp;
use ibc_testkit::context::{MockContext, TendermintContext};
use ibc_testkit::fixtures::core::context::TestContextConfig;
use ibc_testkit::fixtures::core::signer::dummy_account_id;
use ibc_testkit::hosts::{TestBlock, TestHost};
use ibc_testkit::testapp::ibc::clients::mock::client_state::{
    client_type as mock_client_type, MockClientState,
};
use ibc_testkit::testapp::ibc::clients::mock::consensus_state::MockConsensusState;
use ibc_testkit::testapp::ibc::core::types::DEFAULT_BLOCK_TIME_SECS;
use rstest::*;

struct Fixture {
    ctx: TendermintContext,
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
    let latest_timestamp = Timestamp::now();

    let mut ctx_a: TendermintContext = TestContextConfig::builder()
        .latest_timestamp(latest_timestamp)
        .build();

    // create a ctx_b
    let ctx_b: MockContext = TestContextConfig::builder()
        .latest_height(substitute_height)
        .latest_timestamp(latest_timestamp)
        .build();

    let signer = dummy_account_id();

    let subject_client_header = ctx_b
        .host
        .get_block(&subject_height)
        .expect("block exists")
        .into_header();

    let msg = MsgCreateClient::new(
        MockClientState::new(subject_client_header)
            .with_trusting_period(subject_trusting_period)
            .into(),
        MockConsensusState::new(subject_client_header).into(),
        signer.clone(),
    );

    let subject_msg_envelope = MsgEnvelope::from(ClientMsg::from(msg));

    let client_type = mock_client_type();
    let subject_client_id =
        client_type.build_client_id(ctx_a.ibc_store().client_counter().unwrap());

    ctx_a
        .dispatch(subject_msg_envelope)
        .expect("create subject client execution");

    let substitute_client_header = ctx_b
        .host
        .get_block(&substitute_height)
        .expect("block exists")
        .into_header();

    let msg = MsgCreateClient::new(
        MockClientState::new(substitute_client_header)
            .with_trusting_period(substitute_trusting_period)
            .into(),
        MockConsensusState::new(substitute_client_header).into(),
        signer.clone(),
    );

    let substitute_msg_envelope = MsgEnvelope::from(ClientMsg::from(msg));

    let substitute_client_id =
        client_type.build_client_id(ctx_a.ibc_store().client_counter().unwrap());

    ctx_a
        .dispatch(substitute_msg_envelope)
        .expect("create substitute client execution");

    let subject_client_trusted_timestamp =
        (subject_client_header.timestamp() + subject_trusting_period).expect("no error");

    // Let the subject client state expire.
    while ctx_a.latest_timestamp() <= subject_client_trusted_timestamp {
        ctx_a.advance_block_height();
    }

    // at this point, the subject client should be expired.
    // and the substitute client should be active or expired according
    // to the fixture arguments

    Fixture {
        ctx: ctx_a,
        subject_client_id,
        substitute_client_id,
        signer,
    }
}

#[rstest]
fn test_recover_client_ok() {
    // NOTE: The trusting periods are extended by 1 second as clients expire
    // when the elapsed time since a trusted header MATCHES the trusting period.
    let subject_trusting_period = Duration::from_secs(DEFAULT_BLOCK_TIME_SECS + 1);
    let substitute_trusting_period = Duration::from_secs(DEFAULT_BLOCK_TIME_SECS + 1) * 10;
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

    assert_eq!(
        ctx.ibc_store()
            .client_state(&subject_client_id)
            .expect("substitute client state exists")
            .status(ctx.ibc_store(), &subject_client_id)
            .expect("no error"),
        ClientStatus::Expired
    );

    assert_eq!(
        ctx.ibc_store()
            .client_state(&substitute_client_id)
            .expect("substitute client state exists")
            .status(ctx.ibc_store(), &substitute_client_id)
            .expect("no error"),
        ClientStatus::Active
    );

    let msg = MsgRecoverClient {
        subject_client_id,
        substitute_client_id,
        signer,
    };

    recover_client::validate(ctx.ibc_store_mut(), msg.clone())
        .expect("client recovery execution happy path");

    recover_client::execute(ctx.ibc_store_mut(), msg.clone())
        .expect("client recovery execution happy path");

    // client state is copied.
    assert_eq!(
        ctx.ibc_store()
            .client_state(&msg.subject_client_id)
            .unwrap(),
        ctx.ibc_store()
            .client_state(&msg.substitute_client_id)
            .unwrap(),
    );

    // latest consensus state is copied.
    assert_eq!(
        ctx.ibc_store()
            .consensus_state(&ClientConsensusStatePath::new(
                msg.subject_client_id,
                substitute_height.revision_number(),
                substitute_height.revision_height(),
            ))
            .unwrap(),
        ctx.ibc_store()
            .consensus_state(&ClientConsensusStatePath::new(
                msg.substitute_client_id,
                substitute_height.revision_number(),
                substitute_height.revision_height(),
            ))
            .unwrap(),
    );
}

#[rstest]
fn test_recover_client_with_expired_substitute() {
    // twice of DEFAULT_BLOCK_TIME_SECS to make sure the substitute client is expired as well
    let subject_trusting_period = Duration::from_secs(DEFAULT_BLOCK_TIME_SECS) * 2;
    let substitute_trusting_period = Duration::from_secs(DEFAULT_BLOCK_TIME_SECS);
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

    assert_eq!(
        ctx.ibc_store()
            .client_state(&subject_client_id)
            .expect("substitute client state exists")
            .status(ctx.ibc_store(), &subject_client_id)
            .expect("no error"),
        ClientStatus::Expired
    );

    assert_eq!(
        ctx.ibc_store()
            .client_state(&substitute_client_id)
            .expect("substitute client state exists")
            .status(ctx.ibc_store(), &substitute_client_id)
            .expect("no error"),
        ClientStatus::Expired
    );

    let msg = MsgRecoverClient {
        subject_client_id,
        substitute_client_id,
        signer,
    };

    recover_client::validate(ctx.ibc_store(), msg)
        .expect_err("expected client recovery validation to fail");
}

#[rstest]
fn test_recover_client_with_matching_heights() {
    let subject_trusting_period = Duration::from_secs(DEFAULT_BLOCK_TIME_SECS);
    let substitute_trusting_period = Duration::from_secs(DEFAULT_BLOCK_TIME_SECS) * 10;
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

    assert_eq!(
        ctx.ibc_store()
            .client_state(&subject_client_id)
            .expect("substitute client state exists")
            .status(ctx.ibc_store(), &subject_client_id)
            .expect("no error"),
        ClientStatus::Expired
    );

    assert_eq!(
        ctx.ibc_store()
            .client_state(&substitute_client_id)
            .expect("substitute client state exists")
            .status(ctx.ibc_store(), &substitute_client_id)
            .expect("no error"),
        ClientStatus::Active
    );

    let msg = MsgRecoverClient {
        subject_client_id,
        substitute_client_id,
        signer,
    };

    recover_client::validate(ctx.ibc_store(), msg)
        .expect_err("expected client recovery validation to fail");
}
