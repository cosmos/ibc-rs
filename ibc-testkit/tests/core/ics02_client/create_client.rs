use basecoin_store::context::ProvableStore;
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
use ibc::core::commitment_types::merkle::MerkleProof;
use ibc::core::entrypoint::{execute, validate};
use ibc::core::handler::types::error::ContextError;
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::types::identifiers::{ChainId, ClientId};
use ibc::core::host::types::path::{ClientConsensusStatePath, NextClientSequencePath};
use ibc::core::host::{ClientStateRef, ValidationContext};
use ibc_proto::ibc::core::commitment::v1::MerklePath;
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

    let ctx_tm = MockContextConfig::builder()
        .host_id(ChainId::new("tendermint-0").unwrap())
        .latest_height(Height::new(0, 10).expect("no error"))
        .build::<MockContext<TendermintHost>>();

    let ctx_mk = MockContext::<MockHost>::default().with_light_client(
        &client_id,
        LightClientBuilder::init().context(&ctx_tm).build(),
    );

    let client_validation_ctx_mk = ctx_mk.ibc_store().get_client_validation_context();

    let consensus_state_path = ClientConsensusStatePath::new(client_id.clone(), 0, 10);

    let (
        AnyClientState::Tendermint(tm_client_state),
        AnyConsensusState::Tendermint(tm_consensus_state),
    ) = (
        client_validation_ctx_mk
            .client_state(&client_id)
            .expect("client state exists"),
        client_validation_ctx_mk
            .consensus_state(&consensus_state_path)
            .expect("consensus_state exists"),
    )
    else {
        panic!("client and consensus state are not valid")
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
        .store
        .get_proof(
            ctx_tm.latest_height().revision_height().into(),
            &next_client_seq_path.to_string().into(),
        )
        .expect("proof exists");

    let root = tm_consensus_state.inner().root();

    let merkle_path = MerklePath {
        key_path: vec![next_client_seq_path.to_string()],
    };

    let merkle_proof = MerkleProof {
        proofs: vec![proof],
    };

    // with correct value
    merkle_proof
        .verify_membership(
            &tm_client_state.inner().proof_specs,
            root.clone().into(),
            merkle_path.clone(),
            serde_json::to_vec(&next_client_seq_value).expect("valid json serialization"),
            0,
        )
        .expect("proof verification is successful");

    // with incorrect value
    assert!(matches!(
        merkle_proof
            .verify_membership(
                &tm_client_state.inner().proof_specs,
                root.into(),
                merkle_path,
                serde_json::to_vec(&1).expect("valid json serialization"),
                0,
            )
            .expect_err("proof verification fails"),
        CommitmentError::VerificationFailure
    ));
}
