use basecoin_store::impls::InMemoryStore;
use ibc::clients::tendermint::types::{
    client_type as tm_client_type, ConsensusState as TmConsensusState,
};
use ibc::core::client::context::client_state::ClientStateCommon;
use ibc::core::client::context::ClientValidationContext;
use ibc::core::client::types::error::ClientError;
use ibc::core::client::types::msgs::{ClientMsg, MsgCreateClient};
use ibc::core::client::types::Height;
use ibc::core::commitment_types::error::CommitmentError;
use ibc::core::entrypoint::{execute, validate};
use ibc::core::handler::types::error::ContextError;
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::types::identifiers::{ChainId, ClientId};
use ibc::core::host::types::path::{ClientConsensusStatePath, NextClientSequencePath};
use ibc::core::host::{ClientStateRef, ValidationContext};
use ibc_query::core::context::ProvableContext;
use ibc_testkit::context::MockContext;
use ibc_testkit::fixtures::clients::tendermint::{
    dummy_tendermint_header, dummy_tm_client_state_from_header,
};
use ibc_testkit::fixtures::core::context::MockContextConfig;
use ibc_testkit::fixtures::core::signer::dummy_account_id;
use ibc_testkit::hosts::{MockHost, TendermintHost};
use ibc_testkit::testapp::ibc::clients::mock::client_state::{
    client_type as mock_client_type, MockClientState,
};
use ibc_testkit::testapp::ibc::clients::mock::consensus_state::MockConsensusState;
use ibc_testkit::testapp::ibc::clients::mock::header::MockHeader;
use ibc_testkit::testapp::ibc::clients::{AnyClientState, AnyConsensusState};
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::{DefaultIbcStore, LightClientBuilder, MockIbcStore};
use test_log::test;

#[test]
fn test_create_client_ok() {
    let mut ctx = DefaultIbcStore::default();
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

    let expected_client_state =
        ClientStateRef::<DefaultIbcStore>::try_from(msg.client_state).unwrap();
    assert_eq!(expected_client_state.client_type(), client_type);
    assert_eq!(ctx.client_state(&client_id).unwrap(), expected_client_state);
}

#[test]
fn test_tm_create_client_ok() {
    let signer = dummy_account_id();

    let mut ctx = DefaultIbcStore::default();

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

    let expected_client_state =
        ClientStateRef::<MockIbcStore<InMemoryStore>>::try_from(msg.client_state).unwrap();
    assert_eq!(expected_client_state.client_type(), client_type);
    assert_eq!(ctx.client_state(&client_id).unwrap(), expected_client_state);
}

#[test]
fn test_invalid_frozen_tm_client_creation() {
    let signer = dummy_account_id();

    let ctx = DefaultIbcStore::default();

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

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg));

    let res = validate(&ctx, &router, msg_envelope);

    assert!(matches!(
        res,
        Err(ContextError::ClientError(ClientError::ClientFrozen { .. }))
    ))
}

#[test]
fn test_tm_create_client_proof_verification_ok() {
    let client_id = ClientId::new("07-tendermint", 0).expect("no error");
    let client_height = Height::new(0, 10).expect("no error");

    let ctx_tm = MockContextConfig::builder()
        .host_id(ChainId::new("tendermint-0").unwrap())
        .latest_height(client_height)
        .build::<MockContext<TendermintHost>>();

    let ctx_mk = MockContext::<MockHost>::default().with_light_client(
        &client_id,
        LightClientBuilder::init().context(&ctx_tm).build(),
    );

    let client_validation_ctx_mk = ctx_mk.ibc_store().get_client_validation_context();

    let (AnyClientState::Tendermint(tm_client_state),) = (client_validation_ctx_mk
        .client_state(&client_id)
        .expect("client state exists"),)
    else {
        panic!("client state is not valid")
    };

    let latest_client_height = tm_client_state.latest_height();
    let consensus_state_path = ClientConsensusStatePath::new(
        client_id.clone(),
        latest_client_height.revision_number(),
        latest_client_height.revision_height(),
    );

    let AnyConsensusState::Tendermint(tm_consensus_state) = client_validation_ctx_mk
        .consensus_state(&consensus_state_path)
        .expect("consensus_state exists")
    else {
        panic!("consensus state is not valid")
    };

    let next_client_seq_path = NextClientSequencePath;
    let next_client_seq_value = client_validation_ctx_mk
        .client_counter()
        .expect("counter exists");

    assert_eq!(
        next_client_seq_value, 0,
        "client counter is not incremented"
    );

    let proof = ctx_tm
        .ibc_store()
        .get_proof(ctx_tm.latest_height(), &next_client_seq_path.clone().into())
        .expect("proof exists")
        .try_into()
        .expect("value merkle proof");

    let root = tm_consensus_state.inner().root();

    // correct value verification
    tm_client_state
        .verify_membership(
            &ctx_tm.ibc_store().commitment_prefix(),
            &proof,
            &root,
            next_client_seq_path.clone().into(),
            serde_json::to_vec(&next_client_seq_value).expect("valid json serialization"),
        )
        .expect("successful proof verification");

    // incorrect value verification
    assert!(matches!(
        tm_client_state
            .verify_membership(
                &ctx_tm.ibc_store().commitment_prefix(),
                &proof,
                &root,
                next_client_seq_path.into(),
                serde_json::to_vec(&(next_client_seq_value + 1)).expect("valid json serialization"),
            )
            .expect_err("proof verification fails"),
        ClientError::Ics23Verification(CommitmentError::VerificationFailure)
    ));
}
