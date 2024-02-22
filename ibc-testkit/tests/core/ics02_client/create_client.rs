use std::str::FromStr;
use ibc::clients::tendermint::consensus_state::ConsensusState as TmConsensusState;
use ibc::clients::tendermint::types::client_type as tm_client_type;
use ibc::core::client::context::client_state::ClientStateCommon;
use ibc::core::client::types::error::ClientError;
use ibc::core::client::types::msgs::{ClientMsg, MsgCreateClient};
use ibc::core::client::types::Height;
use ibc::core::entrypoint::{execute, validate};
use ibc::core::handler::types::error::ContextError;
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::ValidationContext;
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
use ibc::core::client::context::ClientExecutionContext;
use ibc::core::host::types::identifiers::{ChainId, ClientId};
use ibc::core::host::types::path::ClientConsensusStatePath;
use ibc::primitives::Timestamp;
use ibc_testkit::testapp::ibc::clients::AnyConsensusState;

#[test]
fn test_create_client_ok() {
    let mut ctx = MockContext::default();
    let mut router = MockRouter::new_with_transfer();
    let signer = dummy_account_id();
    let height = Height::new(0, 42).unwrap();
    let client_id = ClientId::from_str("9999-mock-0").unwrap(); // Adjust as necessary
    let mock_header = MockHeader::new_with_timestamp(height, Timestamp::now());
    // Create a mock consensus state
    let consensus_state = MockConsensusState::new(mock_header).into();
    // Construct the path for storing the consensus state
    let consensus_state_path = ClientConsensusStatePath::new(client_id, height.revision_number(), height.revision_height());
    // Store the consensus state in the MockContext
    ctx.store_consensus_state(consensus_state_path, consensus_state).expect("Failed to store consensus state");

    let msg = MsgCreateClient::new(
        MockClientState::new(mock_header).into(),
        MockConsensusState::new(mock_header).into(),
        signer,
    );

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg.clone()));
    let client_type = mock_client_type();
    let client_id = client_type.build_client_id(ctx.client_counter().unwrap());
    let res = validate(&ctx, &router, msg_envelope.clone());

    assert!(res.is_ok(), "validation happy path");

    let res = execute(&mut ctx, &mut router, msg_envelope);

    assert!(res.is_ok(), "execution happy path");

    let expected_client_state = ctx.decode_client_state(msg.client_state).unwrap();
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
    let client_id_clone = client_id.clone(); // Clone client_id before it's moved

    let chain_id = ChainId::from_str(tm_header.chain_id.as_str()).expect("Never fails");
    let height = Height::new(chain_id.revision_number(), u64::from(tm_header.height)).expect("Never fails");

    // Ensure the correct path and consensus state are being used
    let consensus_state_path = ClientConsensusStatePath::new(client_id, height.revision_number(), height.revision_height());

    // Convert the Tendermint header into the correct ConsensusState type
    let tm_consensus_state = TmConsensusState::from(tm_header.clone());
    // Convert the TmConsensusState into AnyConsensusState for storage
    let any_consensus_state: AnyConsensusState = tm_consensus_state.into(); // This should now work with the correct imports and types

    // Store the consensus state in the MockContext
    ctx.store_consensus_state(consensus_state_path, any_consensus_state).expect("Failed to store consensus state");

    let msg = MsgCreateClient::new(
        tm_client_state,
        TmConsensusState::from(tm_header.clone()).into(),
        signer,
    );

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg.clone()));

    let res = validate(&ctx, &router, msg_envelope.clone());

    assert!(res.is_ok(), "tendermint client validation happy path");

    let res = execute(&mut ctx, &mut router, msg_envelope);

    assert!(res.is_ok(), "tendermint client execution happy path");

    let expected_client_state = ctx.decode_client_state(msg.client_state).unwrap();
    assert_eq!(expected_client_state.client_type(), client_type);
    assert_eq!(ctx.client_state(&client_id_clone).unwrap(), expected_client_state);
}

#[test]
fn test_invalid_not_active_tm_client_creation() {
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
        Err(ContextError::ClientError(ClientError::ClientNotActive {..}))
    ))
}
