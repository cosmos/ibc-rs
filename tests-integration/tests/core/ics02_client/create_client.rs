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
use ibc::core::host::types::identifiers::ClientId;
use ibc::core::host::types::path::{ClientConsensusStatePath, NextClientSequencePath};
use ibc::core::host::{ClientStateRef, ValidationContext};
use ibc_core_client_types::Status;
use ibc_query::core::context::ProvableContext;
use ibc_testkit::context::{MockContext, TendermintContext};
use ibc_testkit::fixtures::clients::tendermint::dummy_tm_client_state_from_header;
#[cfg(feature = "serde")]
use ibc_testkit::fixtures::clients::tendermint::{
    dummy_expired_tendermint_header, dummy_valid_tendermint_header,
};
use ibc_testkit::fixtures::core::context::TestContextConfig;
use ibc_testkit::fixtures::core::signer::dummy_account_id;
use ibc_testkit::fixtures::{Expect, Fixture};
use ibc_testkit::testapp::ibc::clients::mock::client_state::{
    client_type as mock_client_type, MockClientState,
};
use ibc_testkit::testapp::ibc::clients::mock::consensus_state::MockConsensusState;
use ibc_testkit::testapp::ibc::clients::mock::header::MockHeader;
use ibc_testkit::testapp::ibc::clients::{AnyClientState, AnyConsensusState};
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::{DefaultIbcStore, LightClientBuilder};
use ibc_testkit::utils::year_2023;
use test_log::test;

enum Ctx {
    Default,
}

#[allow(warnings)]
enum Msg {
    ValidMockHeader,
    ExpiredMockHeader,
    ValidTendermintHeader,
    ExpiredTendermintHeader,
    FrozenTendermintHeader,
    NoHostTimestamp,
}

fn create_client_fixture(ctx_variant: Ctx, msg_variant: Msg) -> Fixture<MsgCreateClient> {
    let ctx = match ctx_variant {
        Ctx::Default => DefaultIbcStore::default(),
    };

    let signer = dummy_account_id();
    let height = Height::new(0, 42).unwrap();

    let msg = match msg_variant {
        Msg::ValidMockHeader => {
            let header = MockHeader::new(height).with_current_timestamp();

            MsgCreateClient::new(
                MockClientState::new(header).into(),
                MockConsensusState::new(header).into(),
                signer,
            )
        }
        Msg::ExpiredMockHeader => {
            let header = MockHeader::new(height).with_timestamp(year_2023());

            MsgCreateClient::new(
                MockClientState::new(header).into(),
                MockConsensusState::new(header).into(),
                signer,
            )
        }
        Msg::ValidTendermintHeader => {
            let tm_header = dummy_valid_tendermint_header();

            MsgCreateClient::new(
                dummy_tm_client_state_from_header(tm_header.clone()).into(),
                TmConsensusState::from(tm_header).into(),
                signer,
            )
        }
        Msg::ExpiredTendermintHeader => {
            let tm_header = dummy_expired_tendermint_header();

            MsgCreateClient::new(
                dummy_tm_client_state_from_header(tm_header.clone()).into(),
                TmConsensusState::from(tm_header).into(),
                signer,
            )
        }
        Msg::FrozenTendermintHeader => {
            let tm_header = dummy_valid_tendermint_header();

            MsgCreateClient::new(
                dummy_tm_client_state_from_header(tm_header.clone())
                    .inner()
                    .clone()
                    .with_frozen_height(Height::min(0))
                    .into(),
                TmConsensusState::from(tm_header).into(),
                signer,
            )
        }
        Msg::NoHostTimestamp => {
            let header = MockHeader::new(height);

            MsgCreateClient::new(
                MockClientState::new(header).into(),
                MockConsensusState::new(header).into(),
                signer,
            )
        }
    };

    Fixture { ctx, msg }
}

fn create_client_validate(fxt: &Fixture<MsgCreateClient>, expect: Expect) {
    let router = MockRouter::new_with_transfer();
    let msg_envelope = MsgEnvelope::from(ClientMsg::from(fxt.msg.clone()));
    let res = validate(&fxt.ctx, &router, msg_envelope);
    let err_msg = fxt.generate_error_msg(&expect, "validation", &res);
    match expect {
        Expect::Failure(err) => match err {
            Some(_e) => {
                assert!(matches!(res, Err(_e)));
            }
            _ => {
                assert!(res.is_err(), "{err_msg}");
            }
        },
        Expect::Success => {
            assert!(res.is_ok(), "{err_msg}");
        }
    }
}

fn create_client_execute(fxt: &mut Fixture<MsgCreateClient>, expect: Expect) {
    let mut router = MockRouter::new_with_transfer();
    let msg_envelope = MsgEnvelope::from(ClientMsg::from(fxt.msg.clone()));
    let expected_client_state =
        ClientStateRef::<DefaultIbcStore>::try_from(fxt.msg.client_state.clone()).unwrap();

    let client_type = match expected_client_state {
        AnyClientState::Mock(_) => mock_client_type(),
        AnyClientState::Tendermint(_) => tm_client_type(),
    };
    let client_id = client_type.build_client_id(fxt.ctx.client_counter().unwrap());
    let res = execute(&mut fxt.ctx, &mut router, msg_envelope);
    let err_msg = fxt.generate_error_msg(&expect, "execution", &res);
    match expect {
        Expect::Failure(_) => {
            assert!(res.is_err(), "{err_msg}")
        }
        Expect::Success => {
            assert_eq!(
                fxt.ctx.client_state(&client_id).unwrap(),
                expected_client_state
            );
            assert!(res.is_ok(), "{err_msg}");
        }
    }
}

#[test]
fn test_create_mock_client_ok() {
    let mut fxt = create_client_fixture(Ctx::Default, Msg::ValidMockHeader);
    create_client_validate(&fxt, Expect::Success);
    create_client_execute(&mut fxt, Expect::Success);
}

#[test]
fn test_create_expired_mock_client() {
    let fxt = create_client_fixture(Ctx::Default, Msg::ExpiredMockHeader);
    create_client_validate(
        &fxt,
        Expect::Failure(Some(ContextError::ClientError(
            ClientError::InvalidStatus {
                actual: Status::Expired,
            },
        ))),
    );
}

#[test]
fn test_create_mock_client_without_timestamp() {
    let fxt = create_client_fixture(Ctx::Default, Msg::NoHostTimestamp);
    create_client_validate(&fxt, Expect::Failure(None));
}

#[cfg(feature = "serde")]
#[test]
fn test_create_tm_client_ok() {
    let mut fxt = create_client_fixture(Ctx::Default, Msg::ValidTendermintHeader);
    create_client_validate(&fxt, Expect::Success);
    create_client_execute(&mut fxt, Expect::Success);
}

#[cfg(feature = "serde")]
#[test]
fn test_create_expired_tm_client() {
    let fxt = create_client_fixture(Ctx::Default, Msg::ExpiredTendermintHeader);
    create_client_validate(
        &fxt,
        Expect::Failure(Some(ContextError::ClientError(
            ClientError::InvalidStatus {
                actual: Status::Expired,
            },
        ))),
    );
}

#[cfg(feature = "serde")]
#[test]
fn test_create_frozen_tm_client() {
    let fxt = create_client_fixture(Ctx::Default, Msg::FrozenTendermintHeader);
    create_client_validate(
        &fxt,
        Expect::Failure(Some(ContextError::ClientError(
            ClientError::InvalidStatus {
                actual: Status::Frozen,
            },
        ))),
    );
}

#[test]
fn test_tm_create_client_proof_verification_ok() {
    let client_id = ClientId::new("07-tendermint", 0).expect("no error");
    let client_height = Height::new(0, 10).expect("no error");

    let ctx_tm = TestContextConfig::builder()
        .latest_height(client_height)
        .build::<TendermintContext>();

    let ctx_mk = MockContext::default().with_light_client(
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
        ClientError::FailedIcs23Verification(CommitmentError::VerificationFailure)
    ));
}
