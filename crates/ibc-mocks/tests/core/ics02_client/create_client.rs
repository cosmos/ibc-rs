use ibc::clients::ics07_tendermint::client_state::ClientState as TmClientState;
use ibc::clients::ics07_tendermint::client_type as tm_client_type;
use ibc::clients::ics07_tendermint::consensus_state::ConsensusState as TmConsensusState;
use ibc::clients::ics07_tendermint::header::test_util::get_dummy_tendermint_header;
use ibc::core::ics02_client::client_state::ClientStateCommon;
use ibc::core::ics02_client::msgs::create_client::MsgCreateClient;
use ibc::core::ics02_client::msgs::ClientMsg;
use ibc::core::ics24_host::identifier::ClientId;
use ibc::core::{execute, validate, MsgEnvelope, ValidationContext};
use ibc::mock::client_state::{client_type as mock_client_type, MockClientState};
use ibc::mock::consensus_state::MockConsensusState;
use ibc::mock::header::MockHeader;
use ibc::test_utils::get_dummy_account_id;
use ibc::Height;
use ibc_mocks::core::definition::MockContext;
use ibc_mocks::router::definition::MockRouter;
use test_log::test;

#[test]
fn test_create_client_ok() {
    let mut ctx = MockContext::default();
    let mut router = MockRouter::new_with_transfer();
    let signer = get_dummy_account_id();
    let height = Height::new(0, 42).unwrap();

    let msg = MsgCreateClient::new(
        MockClientState::new(MockHeader::new(height)).into(),
        MockConsensusState::new(MockHeader::new(height)).into(),
        signer,
    );

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg.clone()));

    let client_type = mock_client_type();

    let client_id = {
        let id_counter = ctx.client_counter().unwrap();
        ClientId::new(client_type.clone(), id_counter).unwrap()
    };

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
    let signer = get_dummy_account_id();

    let mut ctx = MockContext::default();

    let mut router = MockRouter::new_with_transfer();

    let tm_header = get_dummy_tendermint_header();

    let tm_client_state = TmClientState::new_dummy_from_header(tm_header.clone()).into();

    let client_type = tm_client_type();

    let client_id = {
        let id_counter = ctx.client_counter().unwrap();
        ClientId::new(client_type.clone(), id_counter).unwrap()
    };

    let msg = MsgCreateClient::new(
        tm_client_state,
        TmConsensusState::try_from(tm_header).unwrap().into(),
        signer,
    );

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg.clone()));

    let res = validate(&ctx, &router, msg_envelope.clone());

    assert!(res.is_ok(), "tendermint client validation happy path");

    let res = execute(&mut ctx, &mut router, msg_envelope);

    assert!(res.is_ok(), "tendermint client execution happy path");

    let expected_client_state = ctx.decode_client_state(msg.client_state).unwrap();
    assert_eq!(expected_client_state.client_type(), client_type);
    assert_eq!(ctx.client_state(&client_id).unwrap(), expected_client_state);
}
