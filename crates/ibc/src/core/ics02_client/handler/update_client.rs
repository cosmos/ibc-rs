//! Protocol logic specific to processing ICS2 messages of type `MsgUpdateAnyClient`.

use prost::Message;

use crate::core::context::ContextError;
use crate::core::events::{IbcEvent, MessageEvent};
use crate::core::ics02_client::client_state::{
    ClientStateCommon, ClientStateExecution, ClientStateValidation, UpdateKind,
};
use crate::core::ics02_client::error::ClientError;
use crate::core::ics02_client::events::{ClientMisbehaviour, UpdateClient};
use crate::core::ics02_client::msgs::MsgUpdateOrMisbehaviour;
use crate::core::{ExecutionContext, ValidationContext};
use crate::prelude::*;

pub(crate) fn validate<Ctx>(ctx: &Ctx, msg: MsgUpdateOrMisbehaviour) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    ctx.validate_message_signer(msg.signer())?;

    let client_id = msg.client_id().clone();
    let update_kind = match msg {
        MsgUpdateOrMisbehaviour::UpdateClient(_) => UpdateKind::UpdateClient,
        MsgUpdateOrMisbehaviour::Misbehaviour(_) => UpdateKind::SubmitMisbehaviour,
    };

    // Read client state from the host chain store. The client should already exist.
    let client_state = ctx.client_state(&client_id)?;

    {
        let status = client_state.status(ctx.get_client_validation_context(), &client_id)?;
        if !status.is_active() {
            return Err(ClientError::ClientNotActive { status }.into());
        }
    }

    let client_message = msg.client_message();

    client_state.verify_client_message(
        ctx.get_client_validation_context(),
        &client_id,
        client_message,
        &update_kind,
    )?;

    Ok(())
}

pub(crate) fn execute<Ctx>(ctx: &mut Ctx, msg: MsgUpdateOrMisbehaviour) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    let client_id = msg.client_id().clone();
    let update_kind = match msg {
        MsgUpdateOrMisbehaviour::UpdateClient(_) => UpdateKind::UpdateClient,
        MsgUpdateOrMisbehaviour::Misbehaviour(_) => UpdateKind::SubmitMisbehaviour,
    };
    let client_message = msg.client_message();

    let client_state = ctx.client_state(&client_id)?;

    let found_misbehaviour = client_state.check_for_misbehaviour(
        ctx.get_client_validation_context(),
        &client_id,
        client_message.clone(),
        &update_kind,
    )?;

    if found_misbehaviour {
        client_state.update_state_on_misbehaviour(
            ctx.get_client_execution_context(),
            &client_id,
            client_message,
            &update_kind,
        )?;

        let event = IbcEvent::ClientMisbehaviour(ClientMisbehaviour::new(
            client_id,
            client_state.client_type(),
        ));
        ctx.emit_ibc_event(IbcEvent::Message(MessageEvent::Client))?;
        ctx.emit_ibc_event(event)?;
    } else {
        if !matches!(update_kind, UpdateKind::UpdateClient) {
            return Err(ClientError::MisbehaviourHandlingFailure {
                reason: "misbehaviour submitted, but none found".to_string(),
            }
            .into());
        }

        let header = client_message;

        let consensus_heights = client_state.update_state(
            ctx.get_client_execution_context(),
            &client_id,
            header.clone(),
        )?;

        {
            let event = {
                let consensus_height = consensus_heights.get(0).ok_or(ClientError::Other {
                    description: "client update state returned no updated height".to_string(),
                })?;

                IbcEvent::UpdateClient(UpdateClient::new(
                    client_id,
                    client_state.client_type(),
                    *consensus_height,
                    consensus_heights,
                    header.encode_to_vec(),
                ))
            };
            ctx.emit_ibc_event(IbcEvent::Message(MessageEvent::Client))?;
            ctx.emit_ibc_event(event)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use core::ops::Add;
    use core::str::FromStr;
    use core::time::Duration;

    use ibc_proto::google::protobuf::Any;
    use ibc_proto::ibc::lightclients::tendermint::v1::{ClientState as RawTmClientState, Fraction};
    use tendermint_testgen::Validator as TestgenValidator;
    use test_log::test;

    use super::*;
    use crate::clients::ics07_tendermint::client_state::ClientState as TmClientState;
    use crate::clients::ics07_tendermint::client_type as tm_client_type;
    use crate::clients::ics07_tendermint::header::Header as TmHeader;
    use crate::clients::ics07_tendermint::misbehaviour::Misbehaviour as TmMisbehaviour;
    use crate::core::events::IbcEvent;
    use crate::core::ics02_client::client_type::ClientType;
    use crate::core::ics02_client::handler::update_client::{execute, validate};
    use crate::core::ics02_client::msgs::misbehaviour::MsgSubmitMisbehaviour;
    use crate::core::ics02_client::msgs::update_client::MsgUpdateClient;
    use crate::core::ics23_commitment::specs::ProofSpecs;
    use crate::core::ics24_host::identifier::{ChainId, ClientId};
    use crate::core::timestamp::Timestamp;
    use crate::mock::client_state::{client_type as mock_client_type, MockClientState};
    use crate::mock::context::{
        AnyClientState, AnyConsensusState, MockClientConfig, MockContext, MockContextConfig,
    };
    use crate::mock::header::MockHeader;
    use crate::mock::host::{HostBlock, HostType};
    use crate::mock::misbehaviour::Misbehaviour as MockMisbehaviour;
    use crate::test_utils::get_dummy_account_id;
    use crate::{downcast, Height};

    #[test]
    fn test_update_client_ok() {
        let client_id = ClientId::default();
        let signer = get_dummy_account_id();

        let timestamp = Timestamp::now();

        let mut ctx = MockContext::default().with_client(&client_id, Height::new(0, 42).unwrap());
        let height = Height::new(0, 46).unwrap();
        let msg = MsgUpdateClient {
            client_id,
            client_message: MockHeader::new(height).with_timestamp(timestamp).into(),
            signer,
        };

        let res = validate(&ctx, MsgUpdateOrMisbehaviour::UpdateClient(msg.clone()));

        assert!(res.is_ok(), "validation happy path");

        let res = execute(&mut ctx, MsgUpdateOrMisbehaviour::UpdateClient(msg.clone()));
        assert!(res.is_ok(), "execution happy path");

        assert_eq!(
            ctx.client_state(&msg.client_id).unwrap(),
            MockClientState::new(MockHeader::new(height).with_timestamp(timestamp)).into()
        );
    }

    #[test]
    fn test_update_nonexisting_client() {
        let client_id = ClientId::from_str("mockclient1").unwrap();
        let signer = get_dummy_account_id();

        let ctx = MockContext::default().with_client(&client_id, Height::new(0, 42).unwrap());

        let msg = MsgUpdateClient {
            client_id: ClientId::from_str("nonexistingclient").unwrap(),
            client_message: MockHeader::new(Height::new(0, 46).unwrap()).into(),
            signer,
        };

        let res = validate(&ctx, MsgUpdateOrMisbehaviour::UpdateClient(msg));

        assert!(res.is_err());
    }

    #[test]
    fn test_update_synthetic_tendermint_client_adjacent_ok() {
        let client_id = ClientId::new(tm_client_type(), 0).unwrap();
        let client_height = Height::new(1, 20).unwrap();
        let update_height = Height::new(1, 21).unwrap();
        let chain_id_b = ChainId::new("mockgaiaB", 1).unwrap();

        let mut ctx_a = MockContext::new(
            ChainId::new("mockgaiaA", 1).unwrap(),
            HostType::Mock,
            5,
            Height::new(1, 1).unwrap(),
        )
        .with_client_parametrized_with_chain_id(
            chain_id_b.clone(),
            &client_id,
            client_height,
            Some(tm_client_type()), // The target host chain (B) is synthetic TM.
            Some(client_height),
        );

        let ctx_b = MockContext::new(chain_id_b, HostType::SyntheticTendermint, 5, update_height);

        let signer = get_dummy_account_id();

        let mut block = ctx_b.host_block(&update_height).unwrap().clone();
        block.set_trusted_height(client_height);

        let latest_header_height = block.height();
        let msg = MsgUpdateClient {
            client_id,
            client_message: block.into(),
            signer,
        };

        let res = validate(&ctx_a, MsgUpdateOrMisbehaviour::UpdateClient(msg.clone()));
        assert!(res.is_ok());

        let res = execute(
            &mut ctx_a,
            MsgUpdateOrMisbehaviour::UpdateClient(msg.clone()),
        );
        assert!(res.is_ok(), "result: {res:?}");

        let client_state = ctx_a.client_state(&msg.client_id).unwrap();
        assert!(client_state
            .status(&ctx_a, &msg.client_id)
            .unwrap()
            .is_active());
        assert_eq!(client_state.latest_height(), latest_header_height);
    }

    #[test]
    fn test_update_synthetic_tendermint_client_validator_change_ok() {
        let client_id = ClientId::new(tm_client_type(), 0).unwrap();
        let client_height = Height::new(1, 20).unwrap();
        let chain_id_b = ChainId::new("mockgaiaB", 1).unwrap();

        let mut ctx_a = MockContextConfig::builder()
            .host_id(ChainId::new("mockgaiaA", 1).unwrap())
            .latest_height(Height::new(1, 1).unwrap())
            .build()
            .with_client_config(
                // client state initialized with client_height, and
                // [{id: 1, power: 50}, {id: 2, power: 50}] for validator set and next validator set.
                MockClientConfig::builder()
                    .client_chain_id(chain_id_b.clone())
                    .client_id(client_id.clone())
                    .client_state_height(client_height)
                    .client_type(tm_client_type())
                    .build(),
            );

        let ctx_b_val_history = vec![
            // TODO(rano): the validator set params during setups.
            // Here I picked the default validator set which is
            // used at host side client creation.
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

        let signer = get_dummy_account_id();

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

        let res = validate(&ctx_a, MsgUpdateOrMisbehaviour::UpdateClient(msg.clone()));
        assert!(res.is_ok());

        let res = execute(
            &mut ctx_a,
            MsgUpdateOrMisbehaviour::UpdateClient(msg.clone()),
        );
        assert!(res.is_ok(), "result: {res:?}");

        let client_state = ctx_a.client_state(&msg.client_id).unwrap();
        assert!(client_state
            .status(&ctx_a, &msg.client_id)
            .unwrap()
            .is_active());
        assert_eq!(client_state.latest_height(), latest_header_height);
    }

    #[test]
    fn test_update_synthetic_tendermint_client_validator_change_fail() {
        let client_id = ClientId::new(tm_client_type(), 0).unwrap();
        let client_height = Height::new(1, 20).unwrap();
        let chain_id_b = ChainId::new("mockgaiaB", 1).unwrap();

        let ctx_a = MockContextConfig::builder()
            .host_id(ChainId::new("mockgaiaA", 1).unwrap())
            .latest_height(Height::new(1, 1).unwrap())
            .build()
            .with_client_config(
                // client state initialized with client_height, and
                // [{id: 1, power: 50}, {id: 2, power: 50}] for validator set and next validator set.
                MockClientConfig::builder()
                    .client_chain_id(chain_id_b.clone())
                    .client_id(client_id.clone())
                    .client_state_height(client_height)
                    .client_type(tm_client_type())
                    .build(),
            );

        let ctx_b_val_history = vec![
            // TODO(rano): the validator set params during setups.
            // Here I picked the default validator set which is
            // used at host side client creation.
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

        let signer = get_dummy_account_id();

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

        let res = validate(&ctx_a, MsgUpdateOrMisbehaviour::UpdateClient(msg.clone()));
        assert!(res.is_err());
    }

    #[test]
    fn test_update_synthetic_tendermint_client_non_adjacent_ok() {
        let client_id = ClientId::new(tm_client_type(), 0).unwrap();
        let client_height = Height::new(1, 20).unwrap();
        let update_height = Height::new(1, 21).unwrap();
        let chain_id_b = ChainId::new("mockgaiaB", 1).unwrap();

        let mut ctx_a = MockContext::new(
            ChainId::new("mockgaiaA", 1).unwrap(),
            HostType::Mock,
            5,
            Height::new(1, 1).unwrap(),
        )
        .with_client_parametrized_history_with_chain_id(
            chain_id_b.clone(),
            &client_id,
            client_height,
            Some(tm_client_type()), // The target host chain (B) is synthetic TM.
            Some(client_height),
        );

        let ctx_b = MockContext::new(chain_id_b, HostType::SyntheticTendermint, 5, update_height);

        let signer = get_dummy_account_id();

        let mut block = ctx_b.host_block(&update_height).unwrap().clone();
        let trusted_height = client_height.clone().sub(1).unwrap();
        block.set_trusted_height(trusted_height);

        let latest_header_height = block.height();
        let msg = MsgUpdateClient {
            client_id,
            client_message: block.into(),
            signer,
        };

        let res = validate(&ctx_a, MsgUpdateOrMisbehaviour::UpdateClient(msg.clone()));
        assert!(res.is_ok());

        let res = execute(
            &mut ctx_a,
            MsgUpdateOrMisbehaviour::UpdateClient(msg.clone()),
        );
        assert!(res.is_ok(), "result: {res:?}");

        let client_state = ctx_a.client_state(&msg.client_id).unwrap();
        assert!(client_state
            .status(&ctx_a, &msg.client_id)
            .unwrap()
            .is_active());
        assert_eq!(client_state.latest_height(), latest_header_height);
    }

    #[test]
    fn test_update_synthetic_tendermint_client_duplicate_ok() {
        let client_id = ClientId::new(tm_client_type(), 0).unwrap();
        let client_height = Height::new(1, 20).unwrap();

        let ctx_a_chain_id = ChainId::new("mockgaiaA", 1).unwrap();
        let ctx_b_chain_id = ChainId::new("mockgaiaB", 1).unwrap();
        let start_height = Height::new(1, 11).unwrap();

        let mut ctx_a = MockContext::new(ctx_a_chain_id, HostType::Mock, 5, start_height)
            .with_client_parametrized_with_chain_id(
                ctx_b_chain_id.clone(),
                &client_id,
                client_height,
                Some(tm_client_type()), // The target host chain (B) is synthetic TM.
                Some(start_height),
            );

        let ctx_b = MockContext::new(
            ctx_b_chain_id,
            HostType::SyntheticTendermint,
            5,
            client_height,
        );

        let signer = get_dummy_account_id();

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

            let tm_block = downcast!(block.clone() => HostBlock::SyntheticTendermint).unwrap();

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
                    unbonding_period: Some(Duration::from_secs(128000).into()),
                    max_clock_drift: Some(Duration::from_millis(3000).into()),
                    latest_height: Some(
                        Height::new(
                            chain_id.revision_number(),
                            u64::from(tm_block.header().height),
                        )
                        .unwrap()
                        .into(),
                    ),
                    proof_specs: ProofSpecs::default().into(),
                    upgrade_path: Default::default(),
                    frozen_height: None,
                    allow_update_after_expiry: false,
                    allow_update_after_misbehaviour: false,
                };

                let client_state = TmClientState::try_from(raw_client_state).unwrap();

                client_state.into()
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

        let res = validate(&ctx_a, MsgUpdateOrMisbehaviour::UpdateClient(msg.clone()));
        assert!(res.is_ok(), "result: {res:?}");

        let res = execute(
            &mut ctx_a,
            MsgUpdateOrMisbehaviour::UpdateClient(msg.clone()),
        );
        assert!(res.is_ok(), "result: {res:?}");

        let client_state = ctx_a.client_state(&msg.client_id).unwrap();
        assert!(client_state
            .status(&ctx_a, &msg.client_id)
            .unwrap()
            .is_active());
        assert_eq!(client_state.latest_height(), latest_header_height);
        assert_eq!(client_state, ctx_a.latest_client_states(&msg.client_id));
    }

    #[test]
    fn test_update_synthetic_tendermint_client_lower_height() {
        let client_id = ClientId::new(tm_client_type(), 0).unwrap();
        let client_height = Height::new(1, 20).unwrap();

        let client_update_height = Height::new(1, 19).unwrap();

        let chain_start_height = Height::new(1, 11).unwrap();

        let ctx_a = MockContext::new(
            ChainId::new("mockgaiaA", 1).unwrap(),
            HostType::Mock,
            5,
            chain_start_height,
        )
        .with_client_parametrized(
            &client_id,
            client_height,
            Some(tm_client_type()), // The target host chain (B) is synthetic TM.
            Some(client_height),
        );

        let ctx_b = MockContext::new(
            ChainId::new("mockgaiaB", 1).unwrap(),
            HostType::SyntheticTendermint,
            5,
            client_height,
        );

        let signer = get_dummy_account_id();

        let block_ref = ctx_b.host_block(&client_update_height).unwrap();

        let msg = MsgUpdateClient {
            client_id,
            client_message: block_ref.clone().into(),
            signer,
        };

        let res = validate(&ctx_a, MsgUpdateOrMisbehaviour::UpdateClient(msg));
        assert!(res.is_err());
    }

    #[test]
    fn test_update_client_events() {
        let client_id = ClientId::default();
        let signer = get_dummy_account_id();

        let timestamp = Timestamp::now();

        let mut ctx = MockContext::default().with_client(&client_id, Height::new(0, 42).unwrap());
        let height = Height::new(0, 46).unwrap();
        let header: Any = MockHeader::new(height).with_timestamp(timestamp).into();
        let msg = MsgUpdateClient {
            client_id: client_id.clone(),
            client_message: header.clone(),
            signer,
        };

        let res = execute(&mut ctx, MsgUpdateOrMisbehaviour::UpdateClient(msg));
        assert!(res.is_ok());

        assert!(matches!(
            ctx.events[0],
            IbcEvent::Message(MessageEvent::Client)
        ));
        let update_client_event = downcast!(&ctx.events[1] => IbcEvent::UpdateClient).unwrap();

        assert_eq!(update_client_event.client_id(), &client_id);
        assert_eq!(update_client_event.client_type(), &mock_client_type());
        assert_eq!(update_client_event.consensus_height(), &height);
        assert_eq!(update_client_event.consensus_heights(), &vec![height]);
        assert_eq!(update_client_event.header(), &header.encode_to_vec());
    }

    fn ensure_misbehaviour(ctx: &MockContext, client_id: &ClientId, client_type: &ClientType) {
        let client_state = ctx.client_state(client_id).unwrap();

        let status = client_state.status(ctx, client_id).unwrap();
        assert!(status.is_frozen(), "client_state status: {status}");

        // check events
        assert_eq!(ctx.events.len(), 2);
        assert!(matches!(
            ctx.events[0],
            IbcEvent::Message(MessageEvent::Client),
        ));
        let misbehaviour_client_event =
            downcast!(&ctx.events[1] => IbcEvent::ClientMisbehaviour).unwrap();
        assert_eq!(misbehaviour_client_event.client_id(), client_id);
        assert_eq!(misbehaviour_client_event.client_type(), client_type);
    }

    /// Tests misbehaviour handling for the mock client.
    /// Misbehaviour evidence consists of identical headers - mock misbehaviour handler considers it
    /// a valid proof of misbehaviour
    #[test]
    fn test_misbehaviour_client_ok() {
        let client_id = ClientId::default();
        let timestamp = Timestamp::now();
        let height = Height::new(0, 46).unwrap();
        let msg = MsgSubmitMisbehaviour {
            client_id: client_id.clone(),
            misbehaviour: MockMisbehaviour {
                client_id: client_id.clone(),
                header1: MockHeader::new(height).with_timestamp(timestamp),
                header2: MockHeader::new(height).with_timestamp(timestamp),
            }
            .into(),
            signer: get_dummy_account_id(),
        };

        let mut ctx = MockContext::default().with_client(&client_id, Height::new(0, 42).unwrap());

        let res = validate(&ctx, MsgUpdateOrMisbehaviour::Misbehaviour(msg.clone()));
        assert!(res.is_ok());
        let res = execute(&mut ctx, MsgUpdateOrMisbehaviour::Misbehaviour(msg));
        assert!(res.is_ok());

        ensure_misbehaviour(&ctx, &client_id, &mock_client_type());
    }

    /// Tests misbehaviour handling failure for a non-existent client
    #[test]
    fn test_misbehaviour_nonexisting_client() {
        let client_id = ClientId::from_str("mockclient1").unwrap();
        let height = Height::new(0, 46).unwrap();
        let msg = MsgSubmitMisbehaviour {
            client_id: ClientId::from_str("nonexistingclient").unwrap(),
            misbehaviour: MockMisbehaviour {
                client_id: client_id.clone(),
                header1: MockHeader::new(height),
                header2: MockHeader::new(height),
            }
            .into(),
            signer: get_dummy_account_id(),
        };

        let ctx = MockContext::default().with_client(&client_id, Height::new(0, 42).unwrap());
        let res = validate(&ctx, MsgUpdateOrMisbehaviour::Misbehaviour(msg));
        assert!(res.is_err());
    }

    /// Tests misbehaviour handling for the synthetic Tendermint client.
    /// Misbehaviour evidence consists of equivocal headers.
    #[test]
    fn test_misbehaviour_synthetic_tendermint_equivocation() {
        let client_id = ClientId::new(tm_client_type(), 0).unwrap();
        let client_height = Height::new(1, 20).unwrap();
        let misbehaviour_height = Height::new(1, 21).unwrap();
        let chain_id_b = ChainId::new("mockgaiaB", 1).unwrap();

        // Create a mock context for chain-A with a synthetic tendermint light client for chain-B
        let mut ctx_a = MockContext::new(
            ChainId::new("mockgaiaA", 1).unwrap(),
            HostType::Mock,
            5,
            Height::new(1, 1).unwrap(),
        )
        .with_client_parametrized_with_chain_id(
            chain_id_b.clone(),
            &client_id,
            client_height,
            Some(tm_client_type()),
            Some(client_height),
        );

        // Create a mock context for chain-B
        let ctx_b = MockContext::new(
            chain_id_b.clone(),
            HostType::SyntheticTendermint,
            5,
            misbehaviour_height,
        );

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

        let msg = MsgSubmitMisbehaviour {
            client_id: client_id.clone(),
            misbehaviour: TmMisbehaviour::new(client_id.clone(), header1, header2).into(),
            signer: get_dummy_account_id(),
        };

        let res = validate(&ctx_a, MsgUpdateOrMisbehaviour::Misbehaviour(msg.clone()));
        assert!(res.is_ok());
        let res = execute(&mut ctx_a, MsgUpdateOrMisbehaviour::Misbehaviour(msg));
        assert!(res.is_ok());
        ensure_misbehaviour(&ctx_a, &client_id, &tm_client_type());
    }

    #[test]
    fn test_misbehaviour_synthetic_tendermint_bft_time() {
        let client_id = ClientId::new(tm_client_type(), 0).unwrap();
        let client_height = Height::new(1, 20).unwrap();
        let misbehaviour_height = Height::new(1, 21).unwrap();
        let chain_id_b = ChainId::new("mockgaiaB", 1).unwrap();

        // Create a mock context for chain-A with a synthetic tendermint light client for chain-B
        let mut ctx_a = MockContext::new(
            ChainId::new("mockgaiaA", 1).unwrap(),
            HostType::Mock,
            5,
            Height::new(1, 1).unwrap(),
        )
        .with_client_parametrized_with_chain_id(
            chain_id_b.clone(),
            &client_id,
            client_height,
            Some(tm_client_type()),
            Some(client_height),
        );

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
                Timestamp::from_nanoseconds(Timestamp::now().nanoseconds() + 1_000_000_000)
                    .unwrap();
            let mut tm_block = HostBlock::generate_tm_block(
                chain_id_b,
                misbehaviour_height.revision_height(),
                timestamp,
            );
            tm_block.trusted_height = client_height;
            tm_block
        };

        let msg = MsgSubmitMisbehaviour {
            client_id: client_id.clone(),
            misbehaviour: TmMisbehaviour::new(client_id.clone(), header1.into(), header2.into())
                .into(),
            signer: get_dummy_account_id(),
        };

        let res = validate(&ctx_a, MsgUpdateOrMisbehaviour::Misbehaviour(msg.clone()));
        assert!(res.is_ok());
        let res = execute(&mut ctx_a, MsgUpdateOrMisbehaviour::Misbehaviour(msg));
        assert!(res.is_ok());
        ensure_misbehaviour(&ctx_a, &client_id, &tm_client_type());
    }

    #[test]
    fn test_expired_client() {
        let chain_id_b = ChainId::new("mockgaiaB", 1).unwrap();

        let update_height = Height::new(1, 21).unwrap();
        let client_height = update_height.sub(3).unwrap();

        let client_id = ClientId::new(tm_client_type(), 0).unwrap();

        let timestamp = Timestamp::now();

        let mut ctx_a = MockContextConfig::builder()
            .host_id(ChainId::new("mockgaiaA", 1).unwrap())
            .latest_height(Height::new(1, 1).unwrap())
            .latest_timestamp(timestamp)
            .build()
            .with_client_config(
                MockClientConfig::builder()
                    .client_chain_id(chain_id_b.clone())
                    .client_id(client_id.clone())
                    .client_state_height(client_height)
                    .client_type(tm_client_type())
                    .latest_timestamp(timestamp)
                    .build(),
            );

        let mut ctx_b = MockContextConfig::builder()
            .host_id(chain_id_b.clone())
            .host_type(HostType::SyntheticTendermint)
            .latest_height(update_height)
            .latest_timestamp(timestamp)
            .build();

        {
            let client_state = match ctx_a.client_state(&client_id).unwrap() {
                AnyClientState::Tendermint(tm_client_state) => tm_client_state,
                _ => panic!("never fails. not mock client"),
            };

            let client_trusting_period = client_state.trusting_period;

            let future_timestamp = ctx_a
                .host_timestamp()
                .expect("never fails")
                .add(client_trusting_period)
                .expect("overflow");

            ctx_a.advance_host_chain_height_with_timestamp(future_timestamp);
            ctx_b.advance_host_chain_height_with_timestamp(future_timestamp);
        }

        let client_state = ctx_a.client_state(&client_id).unwrap();

        assert!(client_state
            .status(&ctx_a, &client_id)
            .unwrap()
            .is_expired());
    }
}
