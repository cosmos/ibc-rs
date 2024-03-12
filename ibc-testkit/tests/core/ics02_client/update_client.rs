use core::str::FromStr;
use core::time::Duration;

use ibc::clients::tendermint::client_state::ClientState;
use ibc::clients::tendermint::types::proto::v1::{ClientState as RawTmClientState, Fraction};
use ibc::clients::tendermint::types::{
    client_type as tm_client_type, ClientState as TmClientState, Header as TmHeader,
    Misbehaviour as TmMisbehaviour,
};
use ibc::core::client::context::client_state::{ClientStateCommon, ClientStateValidation};
use ibc::core::client::context::ClientValidationContext;
use ibc::core::client::types::msgs::{ClientMsg, MsgUpdateClient};
use ibc::core::client::types::proto::v1::Height as RawHeight;
use ibc::core::client::types::Height;
use ibc::core::commitment_types::specs::ProofSpecs;
use ibc::core::entrypoint::{execute, validate};
use ibc::core::handler::types::events::{IbcEvent, MessageEvent};
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::types::identifiers::{ChainId, ClientId, ClientType};
use ibc::core::host::types::path::ClientConsensusStatePath;
use ibc::core::host::ValidationContext;
use ibc::core::primitives::Timestamp;
use ibc::primitives::proto::Any;
use ibc::primitives::ToVec;
use ibc_testkit::fixtures::core::context::MockContextConfig;
use ibc_testkit::fixtures::core::signer::dummy_account_id;
use ibc_testkit::hosts::block::{HostBlock, HostType};
use ibc_testkit::testapp::ibc::clients::mock::client_state::{
    client_type as mock_client_type, MockClientState,
};
use ibc_testkit::testapp::ibc::clients::mock::header::MockHeader;
use ibc_testkit::testapp::ibc::clients::mock::misbehaviour::Misbehaviour as MockMisbehaviour;
use ibc_testkit::testapp::ibc::clients::AnyConsensusState;
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::{MockClientConfig, MockContext};
use rstest::*;
use tendermint_testgen::Validator as TestgenValidator;

struct Fixture {
    ctx: MockContext,
    router: MockRouter,
}

#[fixture]
fn fixture() -> Fixture {
    let client_id = ClientId::new("07-tendermint", 0).expect("no error");

    let ctx = MockContext::default().with_client_config(
        MockClientConfig::builder()
            .client_id(client_id.clone())
            .latest_height(Height::new(0, 42).unwrap())
            .build(),
    );

    let router = MockRouter::new_with_transfer();

    Fixture { ctx, router }
}

/// Returns a `MsgEnvelope` with the `client_message` field set to a `MockMisbehaviour` report.
fn msg_update_client(client_id: &ClientId) -> MsgEnvelope {
    let timestamp = Timestamp::now();
    let height = Height::new(0, 46).unwrap();
    let msg = MsgUpdateClient {
        client_id: client_id.clone(),
        client_message: MockMisbehaviour {
            client_id: client_id.clone(),
            header1: MockHeader::new(height).with_timestamp(timestamp),
            header2: MockHeader::new(height).with_timestamp(timestamp),
        }
        .into(),
        signer: dummy_account_id(),
    };

    MsgEnvelope::from(ClientMsg::from(msg))
}

#[rstest]
fn test_update_client_ok(fixture: Fixture) {
    let Fixture {
        mut ctx,
        mut router,
    } = fixture;

    let client_id = ClientId::new("07-tendermint", 0).expect("no error");
    let signer = dummy_account_id();
    let timestamp = Timestamp::now();

    let height = Height::new(0, 46).unwrap();
    let msg = MsgUpdateClient {
        client_id,
        client_message: MockHeader::new(height).with_timestamp(timestamp).into(),
        signer,
    };

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg.clone()));

    let res = validate(&ctx, &router, msg_envelope.clone());

    assert!(res.is_ok(), "validation happy path");

    let res = execute(&mut ctx, &mut router, msg_envelope);

    assert!(res.is_ok(), "execution happy path");

    assert_eq!(
        ctx.client_state(&msg.client_id).unwrap(),
        MockClientState::new(MockHeader::new(height).with_timestamp(timestamp)).into()
    );
}

#[rstest]
// Tests successful submission of a header with a height below the latest
// client's height and ensures that `ConsensusState` is stored at the correct
// path (header height).
fn test_update_client_with_prev_header() {
    let client_id = ClientId::new("07-tendermint", 0).expect("no error");
    let chain_id_b = ChainId::new("mockgaiaA-0").unwrap();
    let latest_height = Height::new(0, 42).unwrap();
    let height_1 = Height::new(0, 43).unwrap();
    let height_2 = Height::new(0, 44).unwrap();

    let mut ctx = MockContext::default().with_client_config(
        MockClientConfig::builder()
            .client_chain_id(chain_id_b.clone())
            .client_type(tm_client_type())
            .client_id(client_id.clone())
            .latest_height(latest_height)
            .build(),
    );
    let mut router = MockRouter::new_with_transfer();

    fn build_msg_from_header(
        chain_id: ChainId,
        client_id: ClientId,
        target_height: Height,
        trusted_height: Height,
    ) -> MsgEnvelope {
        let mut tm_block = HostBlock::generate_tm_block(
            chain_id,
            target_height.revision_height(),
            Timestamp::now(),
        );

        tm_block.trusted_height = trusted_height;

        let msg = MsgUpdateClient {
            client_id,
            client_message: TmHeader::from(tm_block).into(),
            signer: dummy_account_id(),
        };

        MsgEnvelope::from(ClientMsg::from(msg))
    }

    let msg_1 = build_msg_from_header(
        chain_id_b.clone(),
        client_id.clone(),
        height_1,
        latest_height,
    );

    let msg_2 = build_msg_from_header(chain_id_b, client_id.clone(), height_2, latest_height);

    // First, submit a header with `height_2` to set the client's latest
    // height to `height_2`.
    let _ = validate(&ctx, &router, msg_2.clone());
    let _ = execute(&mut ctx, &mut router, msg_2);

    // Then, submit a header with `height_1` to see if the client's latest
    // height remains `height_2` and the consensus state is stored at the
    // correct path (`height_1`).
    let _ = validate(&ctx, &router, msg_1.clone());
    let _ = execute(&mut ctx, &mut router, msg_1);

    let client_state = ctx.client_state(&client_id).unwrap();
    assert_eq!(client_state.latest_height(), height_2);

    let cons_state_path = ClientConsensusStatePath::new(
        client_id,
        height_1.revision_number(),
        height_1.revision_height(),
    );
    assert!(ctx.consensus_state(&cons_state_path).is_ok());
}

/// Tests that the Tendermint client consensus state pruning logic
/// functions correctly.
///
/// This test sets up a MockContext with host height 1 and a trusting
/// period of 3 seconds. It then advances the state of the MockContext
/// by 2 heights, and thus 6 seconds, due to the DEFAULT_BLOCK_TIME_SECS
/// constant being set to 3 seconds. At this point, the chain is at height
/// 3. Any consensus states associated with a block more than 3 seconds
/// in the past should be expired and pruned from the IBC store. The test
/// thus checks that the consensus state at height 1 is not contained in
/// the store. It also checks that the consensus state at height 2 is
/// contained in the store and has not expired.
#[rstest]
fn test_consensus_state_pruning() {
    let chain_id = ChainId::new("mockgaiaA-1").unwrap();

    let client_height = Height::new(1, 1).unwrap();

    let client_id = tm_client_type().build_client_id(0);

    let mut ctx = MockContextConfig::builder()
        .host_id(chain_id.clone())
        .host_type(HostType::SyntheticTendermint)
        .latest_height(client_height)
        .latest_timestamp(Timestamp::now())
        .max_history_size(u64::MAX)
        .build()
        .with_client_config(
            MockClientConfig::builder()
                .client_chain_id(chain_id.clone())
                .client_id(client_id.clone())
                .latest_height(client_height)
                .client_type(tm_client_type())
                .trusting_period(Duration::from_secs(3))
                .build(),
        );

    let mut router = MockRouter::new_with_transfer();

    let start_host_timestamp = ctx.host_timestamp().unwrap();

    // Move the chain forward by 2 blocks to pass the trusting period.
    for _ in 1..=2 {
        let signer = dummy_account_id();

        let update_height = ctx.latest_height();

        ctx.advance_host_chain_height();

        let mut block = ctx.host_block(&update_height).unwrap().clone();

        block.set_trusted_height(client_height);

        let msg = MsgUpdateClient {
            client_id: client_id.clone(),
            client_message: block.clone().into(),
            signer,
        };

        let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg));

        let _ = validate(&ctx, &router, msg_envelope.clone());
        let _ = execute(&mut ctx, &mut router, msg_envelope);
    }

    // Check that latest expired consensus state is pruned.
    let expired_height = Height::new(1, 1).unwrap();
    let client_cons_state_path = ClientConsensusStatePath::new(
        client_id.clone(),
        expired_height.revision_number(),
        expired_height.revision_height(),
    );
    assert!(ctx.client_update_meta(&client_id, &expired_height).is_err());
    assert!(ctx.consensus_state(&client_cons_state_path).is_err());

    // Check that latest valid consensus state exists.
    let earliest_valid_height = Height::new(1, 2).unwrap();
    let client_cons_state_path = ClientConsensusStatePath::new(
        client_id.clone(),
        earliest_valid_height.revision_number(),
        earliest_valid_height.revision_height(),
    );

    assert!(ctx
        .client_update_meta(&client_id, &earliest_valid_height)
        .is_ok());
    assert!(ctx.consensus_state(&client_cons_state_path).is_ok());

    let end_host_timestamp = ctx.host_timestamp().unwrap();

    assert_eq!(
        end_host_timestamp,
        (start_host_timestamp + Duration::from_secs(6)).unwrap()
    );
}

#[rstest]
fn test_update_nonexisting_client(fixture: Fixture) {
    let Fixture { ctx, router } = fixture;

    let signer = dummy_account_id();

    let msg = MsgUpdateClient {
        client_id: ClientId::from_str("nonexistingclient").unwrap(),
        client_message: MockHeader::new(Height::new(0, 46).unwrap()).into(),
        signer,
    };

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg));

    let res = validate(&ctx, &router, msg_envelope);

    assert!(res.is_err());
}

#[rstest]
fn test_update_synthetic_tendermint_client_adjacent_ok() {
    let client_id = tm_client_type().build_client_id(0);
    let client_height = Height::new(1, 20).unwrap();
    let update_height = Height::new(1, 21).unwrap();
    let chain_id_b = ChainId::new("mockgaiaB-1").unwrap();

    let mut ctx = MockContextConfig::builder()
        .host_id(ChainId::new("mockgaiaA-1").unwrap())
        .latest_height(Height::new(1, 1).unwrap())
        .build()
        .with_client_config(
            MockClientConfig::builder()
                .client_chain_id(chain_id_b.clone())
                .client_id(client_id.clone())
                .client_type(tm_client_type()) // The target host chain (B) is synthetic TM.
                .latest_height(client_height)
                .build(),
        );

    let mut router = MockRouter::new_with_transfer();

    let ctx_b = MockContextConfig::builder()
        .host_id(chain_id_b)
        .host_type(HostType::SyntheticTendermint)
        .latest_height(update_height)
        .build();

    let signer = dummy_account_id();

    let mut block = ctx_b.host_block(&update_height).unwrap().clone();
    block.set_trusted_height(client_height);

    let latest_header_height = block.height();
    let msg = MsgUpdateClient {
        client_id,
        client_message: block.into(),
        signer,
    };
    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg.clone()));

    let res = validate(&ctx, &router, msg_envelope.clone());
    assert!(res.is_ok());

    let res = execute(&mut ctx, &mut router, msg_envelope);
    assert!(res.is_ok(), "result: {res:?}");

    let client_state = ctx.client_state(&msg.client_id).unwrap();

    assert!(client_state
        .status(&ctx, &msg.client_id)
        .unwrap()
        .is_active());

    assert_eq!(client_state.latest_height(), latest_header_height);
}

#[rstest]
fn test_update_synthetic_tendermint_client_validator_change_ok() {
    let client_id = tm_client_type().build_client_id(0);
    let client_height = Height::new(1, 20).unwrap();
    let chain_id_b = ChainId::new("mockgaiaB-1").unwrap();

    let mut ctx_a = MockContextConfig::builder()
        .host_id(ChainId::new("mockgaiaA-1").unwrap())
        .latest_height(Height::new(1, 1).unwrap())
        .build()
        .with_client_config(
            // client state initialized with client_height, and
            // [{id: 1, power: 50}, {id: 2, power: 50}] for validator set and next validator set.
            MockClientConfig::builder()
                .client_chain_id(chain_id_b.clone())
                .client_id(client_id.clone())
                .latest_height(client_height)
                .client_type(tm_client_type())
                .build(),
        );

    let mut router_a = MockRouter::new_with_transfer();

    let ctx_b_val_history = vec![
        // First two validator sets are default at client creation
        //
        // validator set of height-20
        vec![
            TestgenValidator::new("1").voting_power(50),
            TestgenValidator::new("2").voting_power(50),
        ],
        // validator set of height-21
        vec![
            TestgenValidator::new("1").voting_power(50),
            TestgenValidator::new("2").voting_power(50),
        ],
        // validator set of height-22
        vec![
            TestgenValidator::new("1").voting_power(30),
            TestgenValidator::new("2").voting_power(70),
        ],
        // validator set of height-23
        vec![
            TestgenValidator::new("1").voting_power(20),
            TestgenValidator::new("2").voting_power(80),
        ],
    ];

    let update_height = client_height.add(ctx_b_val_history.len() as u64 - 2);

    let ctx_b = MockContextConfig::builder()
        .host_id(chain_id_b.clone())
        .host_type(HostType::SyntheticTendermint)
        .latest_height(update_height)
        .max_history_size(ctx_b_val_history.len() as u64 - 1)
        .validator_set_history(ctx_b_val_history)
        .build();

    let signer = dummy_account_id();

    let mut block = ctx_b.host_block(&update_height).unwrap().clone();
    block.set_trusted_height(client_height);

    let trusted_next_validator_set = match ctx_b.host_block(&client_height).expect("no error") {
        HostBlock::SyntheticTendermint(header) => header.light_block.next_validators.clone(),
        _ => panic!("unexpected host block type"),
    };

    block.set_trusted_next_validators_set(trusted_next_validator_set);

    let latest_header_height = block.height();
    let msg = MsgUpdateClient {
        client_id,
        client_message: block.into(),
        signer,
    };
    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg.clone()));

    let res = validate(&ctx_a, &router_a, msg_envelope.clone());
    assert!(res.is_ok());

    let res = execute(&mut ctx_a, &mut router_a, msg_envelope);
    assert!(res.is_ok(), "result: {res:?}");

    let client_state = ctx_a.client_state(&msg.client_id).unwrap();
    assert!(client_state
        .status(&ctx_a, &msg.client_id)
        .unwrap()
        .is_active());
    assert_eq!(client_state.latest_height(), latest_header_height);
}

#[rstest]
fn test_update_synthetic_tendermint_client_validator_change_fail() {
    let client_id = tm_client_type().build_client_id(0);
    let client_height = Height::new(1, 20).unwrap();
    let chain_id_b = ChainId::new("mockgaiaB-1").unwrap();

    let ctx_a = MockContextConfig::builder()
        .host_id(ChainId::new("mockgaiaA-1").unwrap())
        .latest_height(Height::new(1, 1).unwrap())
        .build()
        .with_client_config(
            // client state initialized with client_height, and
            // [{id: 1, power: 50}, {id: 2, power: 50}] for validator set and next validator set.
            MockClientConfig::builder()
                .client_chain_id(chain_id_b.clone())
                .client_id(client_id.clone())
                .latest_height(client_height)
                .client_type(tm_client_type())
                .build(),
        );

    let router = MockRouter::new_with_transfer();

    let ctx_b_val_history = vec![
        // First two validator sets are default at client creation
        //
        // validator set of height-20
        vec![
            TestgenValidator::new("1").voting_power(50),
            TestgenValidator::new("2").voting_power(50),
        ],
        // incorrect next validator set for height-20
        // validator set of height-21
        vec![
            TestgenValidator::new("1").voting_power(45),
            TestgenValidator::new("2").voting_power(55),
        ],
        // validator set of height-22
        vec![
            TestgenValidator::new("1").voting_power(30),
            TestgenValidator::new("2").voting_power(70),
        ],
        // validator set of height-23
        vec![
            TestgenValidator::new("1").voting_power(20),
            TestgenValidator::new("2").voting_power(80),
        ],
    ];

    let update_height = client_height.add(ctx_b_val_history.len() as u64 - 2);

    let ctx_b = MockContextConfig::builder()
        .host_id(chain_id_b.clone())
        .host_type(HostType::SyntheticTendermint)
        .latest_height(update_height)
        .max_history_size(ctx_b_val_history.len() as u64 - 1)
        .validator_set_history(ctx_b_val_history)
        .build();

    let signer = dummy_account_id();

    let mut block = ctx_b.host_block(&update_height).unwrap().clone();
    block.set_trusted_height(client_height);

    let trusted_next_validator_set = match ctx_b.host_block(&client_height).expect("no error") {
        HostBlock::SyntheticTendermint(header) => header.light_block.next_validators.clone(),
        _ => panic!("unexpected host block type"),
    };

    block.set_trusted_next_validators_set(trusted_next_validator_set);

    let msg = MsgUpdateClient {
        client_id,
        client_message: block.into(),
        signer,
    };

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg));

    let res = validate(&ctx_a, &router, msg_envelope);

    assert!(res.is_err());
}

#[rstest]
fn test_update_synthetic_tendermint_client_non_adjacent_ok() {
    let client_id = tm_client_type().build_client_id(0);
    let client_height = Height::new(1, 20).unwrap();
    let update_height = Height::new(1, 21).unwrap();
    let chain_id_b = ChainId::new("mockgaiaB-1").unwrap();

    let mut ctx = MockContextConfig::builder()
        .host_id(ChainId::new("mockgaiaA-1").unwrap())
        .latest_height(Height::new(1, 1).unwrap())
        .build()
        .with_client_config(
            MockClientConfig::builder()
                .client_chain_id(chain_id_b.clone())
                .client_id(client_id.clone())
                .client_type(tm_client_type()) // The target host chain (B) is synthetic TM.
                .latest_height(client_height)
                .consensus_state_heights(vec![
                    client_height.sub(1).expect("no error"),
                    client_height,
                ])
                .build(),
        );

    let mut router = MockRouter::new_with_transfer();

    let ctx_b = MockContextConfig::builder()
        .host_id(chain_id_b)
        .host_type(HostType::SyntheticTendermint)
        .latest_height(update_height)
        .build();

    let signer = dummy_account_id();

    let mut block = ctx_b.host_block(&update_height).unwrap().clone();
    let trusted_height = client_height.clone().sub(1).unwrap();
    block.set_trusted_height(trusted_height);

    let latest_header_height = block.height();
    let msg = MsgUpdateClient {
        client_id,
        client_message: block.into(),
        signer,
    };

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg.clone()));

    let res = validate(&ctx, &router, msg_envelope.clone());
    assert!(res.is_ok());

    let res = execute(&mut ctx, &mut router, msg_envelope);
    assert!(res.is_ok(), "result: {res:?}");

    let client_state = ctx.client_state(&msg.client_id).unwrap();

    assert!(client_state
        .status(&ctx, &msg.client_id)
        .unwrap()
        .is_active());

    assert_eq!(client_state.latest_height(), latest_header_height);
}

#[rstest]
fn test_update_synthetic_tendermint_client_duplicate_ok() {
    let client_id = tm_client_type().build_client_id(0);
    let client_height = Height::new(1, 20).unwrap();

    let ctx_a_chain_id = ChainId::new("mockgaiaA-1").unwrap();
    let ctx_b_chain_id = ChainId::new("mockgaiaB-1").unwrap();
    let start_height = Height::new(1, 11).unwrap();

    let mut ctx_a = MockContextConfig::builder()
        .host_id(ctx_a_chain_id)
        .latest_height(start_height)
        .build()
        .with_client_config(
            MockClientConfig::builder()
                .client_chain_id(ctx_b_chain_id.clone())
                .client_id(client_id.clone())
                .client_type(tm_client_type()) // The target host chain (B) is synthetic TM.
                .latest_height(client_height)
                .consensus_state_heights(vec![start_height])
                .build(),
        );

    let mut router_a = MockRouter::new_with_transfer();

    let ctx_b = MockContextConfig::builder()
        .host_id(ctx_b_chain_id)
        .host_type(HostType::SyntheticTendermint)
        .latest_height(client_height)
        .build();

    let signer = dummy_account_id();

    let block = ctx_b.host_block(&client_height).unwrap().clone();

    // Update the trusted height of the header to point to the previous height
    // (`start_height` in this case).
    //
    // Note: The current MockContext interface doesn't allow us to
    // do this without a major redesign.
    let block = match block {
        HostBlock::SyntheticTendermint(mut theader) => {
            // current problem: the timestamp of the new header doesn't match the timestamp of
            // the stored consensus state. If we hack them to match, then commit check fails.
            // FIXME: figure out why they don't match.
            theader.trusted_height = start_height;

            HostBlock::SyntheticTendermint(theader)
        }
        _ => block,
    };

    // Update the client height to `client_height`
    //
    // Note: The current MockContext interface doesn't allow us to
    // do this without a major redesign.
    {
        // FIXME: idea: we need to update the light client with the latest block from
        // chain B
        let consensus_state: AnyConsensusState = block.clone().into();

        let HostBlock::SyntheticTendermint(tm_block) = &block else {
            panic!("unexpected host block type");
        };

        let chain_id = ChainId::from_str(tm_block.header().chain_id.as_str()).unwrap();

        let client_state = {
            #[allow(deprecated)]
            let raw_client_state = RawTmClientState {
                chain_id: chain_id.to_string(),
                trust_level: Some(Fraction {
                    numerator: 1,
                    denominator: 3,
                }),
                trusting_period: Some(Duration::from_secs(64000).into()),
                unbonding_period: Some(Duration::from_secs(128_000).into()),
                max_clock_drift: Some(Duration::from_millis(3000).into()),
                latest_height: Some(
                    Height::new(
                        chain_id.revision_number(),
                        u64::from(tm_block.header().height),
                    )
                    .unwrap()
                    .into(),
                ),
                proof_specs: ProofSpecs::cosmos().into(),
                upgrade_path: Vec::new(),
                frozen_height: Some(RawHeight {
                    revision_number: 0,
                    revision_height: 0,
                }),
                allow_update_after_expiry: false,
                allow_update_after_misbehaviour: false,
            };

            let client_state = TmClientState::try_from(raw_client_state).unwrap();

            ClientState::from(client_state).into()
        };

        let mut ibc_store = ctx_a.ibc_store.lock();
        let client_record = ibc_store.clients.get_mut(&client_id).unwrap();

        client_record
            .consensus_states
            .insert(client_height, consensus_state);

        client_record.client_state = Some(client_state);
    }

    let latest_header_height = block.height();
    let msg = MsgUpdateClient {
        client_id,
        client_message: block.into(),
        signer,
    };

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg.clone()));

    let res = validate(&ctx_a, &router_a, msg_envelope.clone());
    assert!(res.is_ok(), "result: {res:?}");

    let res = execute(&mut ctx_a, &mut router_a, msg_envelope);
    assert!(res.is_ok(), "result: {res:?}");

    let client_state = ctx_a.client_state(&msg.client_id).unwrap();
    assert!(client_state
        .status(&ctx_a, &msg.client_id)
        .unwrap()
        .is_active());
    assert_eq!(client_state.latest_height(), latest_header_height);
    assert_eq!(client_state, ctx_a.latest_client_states(&msg.client_id));
}

#[rstest]
fn test_update_synthetic_tendermint_client_lower_height() {
    let client_id = tm_client_type().build_client_id(0);
    let client_height = Height::new(1, 20).unwrap();

    let client_update_height = Height::new(1, 19).unwrap();

    let chain_start_height = Height::new(1, 11).unwrap();

    let ctx = MockContextConfig::builder()
        .host_id(ChainId::new("mockgaiaA-1").unwrap())
        .latest_height(chain_start_height)
        .build()
        .with_client_config(
            MockClientConfig::builder()
                .client_id(client_id.clone())
                .client_type(tm_client_type()) // The target host chain (B) is synthetic TM.
                .latest_height(client_height)
                .build(),
        );

    let router = MockRouter::new_with_transfer();

    let ctx_b = MockContextConfig::builder()
        .host_id(ChainId::new("mockgaiaB-1").unwrap())
        .host_type(HostType::SyntheticTendermint)
        .latest_height(client_height)
        .build();

    let signer = dummy_account_id();

    let block_ref = ctx_b.host_block(&client_update_height).unwrap();

    let msg = MsgUpdateClient {
        client_id,
        client_message: block_ref.clone().into(),
        signer,
    };

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg));

    let res = validate(&ctx, &router, msg_envelope);
    assert!(res.is_err());
}

#[rstest]
fn test_update_client_events(fixture: Fixture) {
    let Fixture {
        mut ctx,
        mut router,
    } = fixture;

    let client_id = ClientId::new("07-tendermint", 0).expect("no error");
    let signer = dummy_account_id();
    let timestamp = Timestamp::now();

    let height = Height::new(0, 46).unwrap();
    let header: Any = MockHeader::new(height).with_timestamp(timestamp).into();
    let msg = MsgUpdateClient {
        client_id: client_id.clone(),
        client_message: header.clone(),
        signer,
    };
    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg));

    let res = execute(&mut ctx, &mut router, msg_envelope);
    assert!(res.is_ok());

    let ibc_events = ctx.get_events();

    assert!(matches!(
        ibc_events[0],
        IbcEvent::Message(MessageEvent::Client)
    ));
    let IbcEvent::UpdateClient(update_client_event) = &ibc_events[1] else {
        panic!("unexpected event variant");
    };

    assert_eq!(update_client_event.client_id(), &client_id);
    assert_eq!(update_client_event.client_type(), &mock_client_type());
    assert_eq!(update_client_event.consensus_height(), &height);
    assert_eq!(update_client_event.consensus_heights(), &vec![height]);
    assert_eq!(update_client_event.header(), &header.to_vec());
}

fn ensure_misbehaviour(ctx: &MockContext, client_id: &ClientId, client_type: &ClientType) {
    let client_state = ctx.client_state(client_id).unwrap();

    let status = client_state.status(ctx, client_id).unwrap();
    assert!(status.is_frozen(), "client_state status: {status}");

    // check events
    let ibc_events = ctx.get_events();
    assert_eq!(ibc_events.len(), 2);
    assert!(matches!(
        ibc_events[0],
        IbcEvent::Message(MessageEvent::Client),
    ));
    let IbcEvent::ClientMisbehaviour(misbehaviour_client_event) = &ibc_events[1] else {
        panic!("unexpected event variant");
    };
    assert_eq!(misbehaviour_client_event.client_id(), client_id);
    assert_eq!(misbehaviour_client_event.client_type(), client_type);
}

/// Tests misbehaviour handling for the mock client.
///
/// Misbehaviour evidence consists of identical headers - mock misbehaviour handler
/// considers it a valid proof of misbehaviour.
#[rstest]
fn test_misbehaviour_client_ok(fixture: Fixture) {
    let Fixture {
        mut ctx,
        mut router,
    } = fixture;

    let client_id = ClientId::new("07-tendermint", 0).expect("no error");
    let msg_envelope = msg_update_client(&client_id);

    let res = validate(&ctx, &router, msg_envelope.clone());
    assert!(res.is_ok());

    let res = execute(&mut ctx, &mut router, msg_envelope);
    assert!(res.is_ok());

    ensure_misbehaviour(&ctx, &client_id, &mock_client_type());
}

#[rstest]
fn test_submit_misbehaviour_nonexisting_client(fixture: Fixture) {
    let Fixture { router, .. } = fixture;

    let client_id = ClientId::from_str("mockclient1").unwrap();

    let msg_envelope = msg_update_client(&ClientId::from_str("nonexistingclient").unwrap());

    let ctx = MockContext::default().with_client_config(
        MockClientConfig::builder()
            .client_id(client_id.clone())
            .latest_height(Height::new(0, 42).unwrap())
            .build(),
    );
    let res = validate(&ctx, &router, msg_envelope);
    assert!(res.is_err());
}

#[rstest]
fn test_client_update_misbehaviour_nonexisting_client(fixture: Fixture) {
    let Fixture { router, .. } = fixture;

    let client_id = ClientId::from_str("mockclient1").unwrap();

    let msg_envelope = msg_update_client(&ClientId::from_str("nonexistingclient").unwrap());

    let ctx = MockContext::default().with_client_config(
        MockClientConfig::builder()
            .client_id(client_id.clone())
            .latest_height(Height::new(0, 42).unwrap())
            .build(),
    );
    let res = validate(&ctx, &router, msg_envelope);
    assert!(res.is_err());
}

/// Tests misbehaviour handling for the synthetic Tendermint client.
/// Misbehaviour evidence consists of equivocal headers.
#[rstest]
fn test_misbehaviour_synthetic_tendermint_equivocation() {
    let client_id = tm_client_type().build_client_id(0);
    let client_height = Height::new(1, 20).unwrap();
    let misbehaviour_height = Height::new(1, 21).unwrap();
    let chain_id_b = ChainId::new("mockgaiaB-1").unwrap();

    // Create a mock context for chain-A with a synthetic tendermint light client for chain-B
    let mut ctx_a = MockContextConfig::builder()
        .host_id(ChainId::new("mockgaiaA-1").unwrap())
        .latest_height(Height::new(1, 1).unwrap())
        .build()
        .with_client_config(
            MockClientConfig::builder()
                .client_chain_id(chain_id_b.clone())
                .client_id(client_id.clone())
                .client_type(tm_client_type())
                .latest_height(client_height)
                .build(),
        );

    let mut router_a = MockRouter::new_with_transfer();

    // Create a mock context for chain-B
    let ctx_b = MockContextConfig::builder()
        .host_id(chain_id_b.clone())
        .host_type(HostType::SyntheticTendermint)
        .latest_height(misbehaviour_height)
        .build();

    // Get chain-B's header at `misbehaviour_height`
    let header1: TmHeader = {
        let mut block = ctx_b.host_block(&misbehaviour_height).unwrap().clone();
        block.set_trusted_height(client_height);
        block.try_into_tm_block().unwrap().into()
    };

    // Generate an equivocal header for chain-B at `misbehaviour_height`
    let header2 = {
        let mut tm_block = HostBlock::generate_tm_block(
            chain_id_b,
            misbehaviour_height.revision_height(),
            Timestamp::now(),
        );
        tm_block.trusted_height = client_height;
        tm_block.into()
    };

    let msg = MsgUpdateClient {
        client_id: client_id.clone(),
        client_message: TmMisbehaviour::new(client_id.clone(), header1, header2).into(),
        signer: dummy_account_id(),
    };
    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg));

    let res = validate(&ctx_a, &router_a, msg_envelope.clone());
    assert!(res.is_ok());
    let res = execute(&mut ctx_a, &mut router_a, msg_envelope);
    assert!(res.is_ok());
    ensure_misbehaviour(&ctx_a, &client_id, &tm_client_type());
}

#[rstest]
fn test_misbehaviour_synthetic_tendermint_bft_time() {
    let client_id = tm_client_type().build_client_id(0);
    let client_height = Height::new(1, 20).unwrap();
    let misbehaviour_height = Height::new(1, 21).unwrap();
    let chain_id_b = ChainId::new("mockgaiaB-1").unwrap();

    // Create a mock context for chain-A with a synthetic tendermint light client for chain-B
    let mut ctx_a = MockContextConfig::builder()
        .host_id(ChainId::new("mockgaiaA-1").unwrap())
        .latest_height(Height::new(1, 1).unwrap())
        .build()
        .with_client_config(
            MockClientConfig::builder()
                .client_chain_id(chain_id_b.clone())
                .client_id(client_id.clone())
                .client_type(tm_client_type())
                .latest_height(client_height)
                .build(),
        );

    let mut router_a = MockRouter::new_with_transfer();

    // Generate `header1` for chain-B
    let header1 = {
        let mut tm_block = HostBlock::generate_tm_block(
            chain_id_b.clone(),
            misbehaviour_height.revision_height(),
            Timestamp::now(),
        );
        tm_block.trusted_height = client_height;
        tm_block
    };

    // Generate `header2` for chain-B which is identical to `header1` but with a conflicting
    // timestamp
    let header2 = {
        let timestamp =
            Timestamp::from_nanoseconds(Timestamp::now().nanoseconds() + 1_000_000_000).unwrap();
        let mut tm_block = HostBlock::generate_tm_block(
            chain_id_b,
            misbehaviour_height.revision_height(),
            timestamp,
        );
        tm_block.trusted_height = client_height;
        tm_block
    };

    let msg = MsgUpdateClient {
        client_id: client_id.clone(),
        client_message: TmMisbehaviour::new(client_id.clone(), header1.into(), header2.into())
            .into(),
        signer: dummy_account_id(),
    };

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg));

    let res = validate(&ctx_a, &router_a, msg_envelope.clone());
    assert!(res.is_ok());
    let res = execute(&mut ctx_a, &mut router_a, msg_envelope);
    assert!(res.is_ok());
    ensure_misbehaviour(&ctx_a, &client_id, &tm_client_type());
}

#[rstest]
fn test_expired_client() {
    let chain_id_b = ChainId::new("mockgaiaB-1").unwrap();

    let update_height = Height::new(1, 21).unwrap();
    let client_height = update_height.sub(3).unwrap();

    let client_id = tm_client_type().build_client_id(0);

    let timestamp = Timestamp::now();

    let trusting_period = Duration::from_secs(64);

    let mut ctx = MockContextConfig::builder()
        .host_id(ChainId::new("mockgaiaA-1").unwrap())
        .latest_height(Height::new(1, 1).unwrap())
        .latest_timestamp(timestamp)
        .build()
        .with_client_config(
            MockClientConfig::builder()
                .client_chain_id(chain_id_b.clone())
                .client_id(client_id.clone())
                .latest_height(client_height)
                .client_type(tm_client_type())
                .latest_timestamp(timestamp)
                .trusting_period(trusting_period)
                .build(),
        );

    while ctx.host_timestamp().expect("no error") < (timestamp + trusting_period).expect("no error")
    {
        ctx.advance_host_chain_height();
    }

    let client_state = ctx.client_state(&client_id).unwrap();

    assert!(client_state.status(&ctx, &client_id).unwrap().is_expired());
}

#[rstest]
fn test_client_update_max_clock_drift() {
    let chain_id_b = ChainId::new("mockgaiaB-1").unwrap();

    let client_height = Height::new(1, 20).unwrap();

    let client_id = tm_client_type().build_client_id(0);

    let timestamp = Timestamp::now();

    let max_clock_drift = Duration::from_secs(64);

    let ctx_a = MockContextConfig::builder()
        .host_id(ChainId::new("mockgaiaA-1").unwrap())
        .latest_height(Height::new(1, 1).unwrap())
        .latest_timestamp(timestamp)
        .build()
        .with_client_config(
            MockClientConfig::builder()
                .client_chain_id(chain_id_b.clone())
                .client_id(client_id.clone())
                .latest_height(client_height)
                .client_type(tm_client_type())
                .latest_timestamp(timestamp)
                .max_clock_drift(max_clock_drift)
                .build(),
        );

    let router_a = MockRouter::new_with_transfer();

    let mut ctx_b = MockContextConfig::builder()
        .host_id(chain_id_b.clone())
        .host_type(HostType::SyntheticTendermint)
        .latest_height(client_height)
        .latest_timestamp(timestamp)
        .max_history_size(u64::MAX)
        .build();

    while ctx_b.host_timestamp().expect("no error")
        < (ctx_a.host_timestamp().expect("no error") + max_clock_drift).expect("no error")
    {
        ctx_b.advance_host_chain_height();
    }

    // include current block
    ctx_b.advance_host_chain_height();

    let update_height = ctx_b.latest_height();

    let signer = dummy_account_id();

    let mut block = ctx_b.host_block(&update_height).unwrap().clone();
    block.set_trusted_height(client_height);

    let trusted_next_validator_set = match ctx_b.host_block(&client_height).expect("no error") {
        HostBlock::SyntheticTendermint(header) => header.light_block.next_validators.clone(),
        _ => panic!("unexpected host block type"),
    };

    block.set_trusted_next_validators_set(trusted_next_validator_set);

    let msg = MsgUpdateClient {
        client_id,
        client_message: block.clone().into(),
        signer,
    };

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg));

    let res = validate(&ctx_a, &router_a, msg_envelope);
    assert!(res.is_err());
}
