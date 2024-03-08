use ibc::clients::tendermint::types::{
    client_type as tm_client_type, ConsensusState as TmConsensusState,
};
use ibc::core::client::context::client_state::ClientStateCommon;
use ibc::core::client::context::ClientValidationContext;
use ibc::core::client::types::error::ClientError;
use ibc::core::client::types::msgs::{ClientMsg, MsgCreateClient};
use ibc::core::client::types::Height;
use ibc::core::entrypoint::{execute, validate};
use ibc::core::handler::types::error::ContextError;
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::{ClientStateRef, ValidationContext};
use ibc_testkit::fixtures::clients::tendermint::{
    dummy_tendermint_header, dummy_tm_client_state_from_header,
};
use ibc_testkit::fixtures::core::signer::dummy_account_id;
use ibc_testkit::testapp::ibc::clients::mock::client_state::{
    client_type as mock_client_type, MockClientState,
};
use ibc_testkit::testapp::ibc::clients::mock::consensus_state::MockConsensusState;
use ibc_testkit::testapp::ibc::clients::mock::header::MockHeader;
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::MockContext;
use test_log::test;

#[test]
fn test_create_client_ok() {
    let mut ctx = MockContext::default();
    let mut router = MockRouter::new_with_transfer();
    let signer = dummy_account_id();
    let height = Height::new(0, 42).unwrap();

    let msg = MsgCreateClient::new(
        MockClientState::new(MockHeader::new(height)).into(),
        MockConsensusState::new(MockHeader::new(height)).into(),
        signer,
    );

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg.clone()));

    let client_type = mock_client_type();
    let client_id = client_type.build_client_id(ctx.client_counter().unwrap());

    let res = validate(&ctx, &router, msg_envelope.clone());

    assert!(res.is_ok(), "validation happy path");

    let res = execute(&mut ctx, &mut router, msg_envelope);

    assert!(res.is_ok(), "execution happy path");

    let expected_client_state = ClientStateRef::<MockContext>::try_from(msg.client_state).unwrap();
    assert_eq!(expected_client_state.client_type(), client_type);
    assert_eq!(ctx.client_state(&client_id).unwrap(), expected_client_state);
}

#[test]
fn test_tm_create_client_ok() {
    let signer = dummy_account_id();

    let mut ctx = MockContext::default();

    let mut router = MockRouter::new_with_transfer();

    let tm_header = dummy_tendermint_header();

    let tm_client_state = dummy_tm_client_state_from_header(tm_header.clone()).into();

    let client_type = tm_client_type();
    let client_id = client_type.build_client_id(ctx.client_counter().unwrap());

    let msg = MsgCreateClient::new(
        tm_client_state,
        TmConsensusState::from(tm_header).into(),
        signer,
    );

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg.clone()));

    let res = validate(&ctx, &router, msg_envelope.clone());

    assert!(res.is_ok(), "tendermint client validation happy path");

    let res = execute(&mut ctx, &mut router, msg_envelope);

    assert!(res.is_ok(), "tendermint client execution happy path");

    let expected_client_state = ClientStateRef::<MockContext>::try_from(msg.client_state).unwrap();
    assert_eq!(expected_client_state.client_type(), client_type);
    assert_eq!(ctx.client_state(&client_id).unwrap(), expected_client_state);
}

#[test]
fn test_invalid_frozen_tm_client_creation() {
    let signer = dummy_account_id();

    let ctx = MockContext::default();

    let router = MockRouter::new_with_transfer();

    let tm_header = dummy_tendermint_header();

    let tm_client_state = dummy_tm_client_state_from_header(tm_header.clone())
        .inner()
        .clone()
        .with_frozen_height(Height::min(0));

    let msg = MsgCreateClient::new(
        tm_client_state.into(),
        TmConsensusState::from(tm_header).into(),
        signer,
    );

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg.clone()));

    let res = validate(&ctx, &router, msg_envelope.clone());

    assert!(matches!(
        res,
        Err(ContextError::ClientError(ClientError::ClientFrozen { .. }))
    ))
}
