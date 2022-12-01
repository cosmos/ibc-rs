//! Protocol logic specific to processing ICS2 messages of type `MsgSubmitMisbehaviour`.
//!
use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics02_client::context::ClientReader;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics02_client::events::ClientMisbehaviour;
use crate::core::ics02_client::handler::ClientResult;
use crate::core::ics02_client::msgs::misbehaviour::MsgSubmitMisbehaviour;
use crate::core::ics24_host::identifier::ClientId;
use crate::events::IbcEvent;
use crate::handler::{HandlerOutput, HandlerResult};
use crate::prelude::*;

/// The result following the successful processing of a `MsgSubmitMisbehaviour` message.
#[derive(Clone, Debug, PartialEq)]
pub struct MisbehaviourResult {
    pub client_id: ClientId,
    pub client_state: Box<dyn ClientState>,
}

pub fn process(
    ctx: &dyn ClientReader,
    msg: MsgSubmitMisbehaviour,
) -> HandlerResult<ClientResult, ClientError> {
    let mut output = HandlerOutput::builder();

    let MsgSubmitMisbehaviour {
        client_id,
        misbehaviour,
        signer: _,
    } = msg;

    // Read client state from the host chain store.
    let client_state = ctx.client_state(&client_id)?;

    if client_state.is_frozen() {
        return Err(ClientError::ClientFrozen { client_id });
    }

    let client_state = client_state
        .check_misbehaviour_and_update_state(ctx, client_id.clone(), misbehaviour)
        .map_err(|e| ClientError::MisbehaviourHandlingFailure {
            reason: e.to_string(),
        })?;

    output.emit(IbcEvent::ClientMisbehaviour(ClientMisbehaviour::new(
        client_id.clone(),
        client_state.client_type(),
    )));

    let result = ClientResult::Misbehaviour(MisbehaviourResult {
        client_id,
        client_state,
    });

    Ok(output.with_result(result))
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;
    use test_log::test;

    use crate::clients::ics07_tendermint::client_type as tm_client_type;
    use crate::clients::ics07_tendermint::header::Header as TmHeader;
    use crate::clients::ics07_tendermint::misbehaviour::Misbehaviour as TmMisbehaviour;
    use crate::core::ics02_client::client_state::ClientState;
    use crate::core::ics02_client::error::ClientError;
    use crate::core::ics02_client::handler::dispatch;
    use crate::core::ics02_client::handler::ClientResult;
    use crate::core::ics02_client::msgs::misbehaviour::MsgSubmitMisbehaviour;
    use crate::core::ics02_client::msgs::ClientMsg;
    use crate::core::ics24_host::identifier::{ChainId, ClientId};
    use crate::events::IbcEvent;
    use crate::handler::HandlerOutput;
    use crate::mock::client_state::client_type as mock_client_type;
    use crate::mock::client_state::MockClientState;
    use crate::mock::context::MockContext;
    use crate::mock::header::MockHeader;
    use crate::mock::host::{HostBlock, HostType};
    use crate::mock::misbehaviour::Misbehaviour;
    use crate::test_utils::get_dummy_account_id;
    use crate::timestamp::Timestamp;
    use crate::Height;
    use crate::{downcast, prelude::*};

    #[test]
    fn test_misbehaviour_client_ok() {
        let client_id = ClientId::default();
        let signer = get_dummy_account_id();

        let timestamp = Timestamp::now();

        let ctx = MockContext::default().with_client(&client_id, Height::new(0, 42).unwrap());
        let height = Height::new(0, 46).unwrap();
        let msg = MsgSubmitMisbehaviour {
            client_id: client_id.clone(),
            misbehaviour: Misbehaviour {
                client_id: client_id.clone(),
                header1: MockHeader::new(height).with_timestamp(timestamp),
                header2: MockHeader::new(height).with_timestamp(timestamp),
            }
            .into(),
            signer,
        };

        let output = dispatch(&ctx, ClientMsg::Misbehaviour(msg));

        match output {
            Ok(HandlerOutput {
                result,
                events: _,
                log,
            }) => {
                assert!(log.is_empty());
                // Check the result
                match result {
                    ClientResult::Misbehaviour(upd_res) => {
                        assert_eq!(upd_res.client_id, client_id);
                        assert_eq!(
                            upd_res.client_state,
                            MockClientState::new(MockHeader::new(height).with_timestamp(timestamp))
                                .with_frozen_height(Height::new(0, 1).unwrap())
                                .into_box()
                        )
                    }
                    _ => panic!("misbehaviour handler result has incorrect type"),
                }
            }
            Err(err) => {
                panic!("unexpected error: {}", err);
            }
        }
    }

    #[test]
    fn test_misbehaviour_nonexisting_client() {
        let client_id = ClientId::from_str("mockclient1").unwrap();
        let signer = get_dummy_account_id();

        let ctx = MockContext::default().with_client(&client_id, Height::new(0, 42).unwrap());

        let msg = MsgSubmitMisbehaviour {
            client_id: ClientId::from_str("nonexistingclient").unwrap(),
            misbehaviour: Misbehaviour {
                client_id,
                header1: MockHeader::new(Height::new(0, 46).unwrap()),
                header2: MockHeader::new(Height::new(0, 46).unwrap()),
            }
            .into(),
            signer,
        };

        let output = dispatch(&ctx, ClientMsg::Misbehaviour(msg.clone()));

        match output {
            Err(ClientError::ClientNotFound { client_id }) => {
                assert_eq!(client_id, msg.client_id);
            }
            _ => {
                panic!("expected ClientNotFound error, instead got {:?}", output)
            }
        }
    }

    #[test]
    fn test_misbehaviour_client_ok_multiple() {
        let client_ids = vec![
            ClientId::from_str("mockclient1").unwrap(),
            ClientId::from_str("mockclient2").unwrap(),
            ClientId::from_str("mockclient3").unwrap(),
        ];
        let signer = get_dummy_account_id();
        let initial_height = Height::new(0, 45).unwrap();
        let misbehaviour_height = Height::new(0, 49).unwrap();

        let mut ctx = MockContext::default();

        for cid in &client_ids {
            ctx = ctx.with_client(cid, initial_height);
        }

        for client_id in &client_ids {
            let msg = MsgSubmitMisbehaviour {
                client_id: client_id.clone(),
                misbehaviour: Misbehaviour {
                    client_id: client_id.clone(),
                    header1: MockHeader::new(misbehaviour_height),
                    header2: MockHeader::new(misbehaviour_height),
                }
                .into(),
                signer: signer.clone(),
            };

            let output = dispatch(&ctx, ClientMsg::Misbehaviour(msg.clone()));

            match output {
                Ok(HandlerOutput {
                    result: _,
                    events: _,
                    log,
                }) => {
                    assert!(log.is_empty());
                }
                Err(err) => {
                    panic!("unexpected error: {}", err);
                }
            }
        }
    }

    #[test]
    fn test_misbehaviour_synthetic_tendermint_equivocation() {
        let client_id = ClientId::new(tm_client_type(), 0).unwrap();
        let client_height = Height::new(1, 20).unwrap();
        let misbehaviour_height = Height::new(1, 21).unwrap();
        let chain_id_b = ChainId::new("mockgaiaB".to_string(), 1);

        let ctx = MockContext::new(
            ChainId::new("mockgaiaA".to_string(), 1),
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

        let ctx_b = MockContext::new(
            chain_id_b.clone(),
            HostType::SyntheticTendermint,
            5,
            misbehaviour_height,
        );

        let signer = get_dummy_account_id();

        let header1: TmHeader = {
            let mut block = ctx_b.host_block(misbehaviour_height).unwrap().clone();
            block.set_trusted_height(client_height);
            block.try_into_tm_block().unwrap().into()
        };

        let header2 = {
            let mut tm_block = HostBlock::generate_tm_block(
                chain_id_b.clone(),
                misbehaviour_height.revision_height(),
                Timestamp::now(),
            );
            tm_block.trusted_height = client_height;
            tm_block.into()
        };

        let msg = MsgSubmitMisbehaviour {
            client_id: client_id.clone(),
            misbehaviour: TmMisbehaviour::new(client_id.clone(), header1, header2)
                .unwrap()
                .into(),
            signer,
        };

        let output = dispatch(&ctx, ClientMsg::Misbehaviour(msg));

        match output {
            Ok(HandlerOutput {
                result,
                events: _,
                log,
            }) => {
                assert!(log.is_empty());
                // Check the result
                match result {
                    ClientResult::Misbehaviour(upd_res) => {
                        assert!(upd_res.client_state.is_frozen())
                    }
                    _ => panic!("misbehaviour handler result has incorrect type"),
                }
            }
            Err(err) => {
                panic!("unexpected error: {}", err);
            }
        }
    }

    #[test]
    fn test_misbehaviour_synthetic_tendermint_bft_time() {
        let client_id = ClientId::new(tm_client_type(), 0).unwrap();
        let client_height = Height::new(1, 20).unwrap();
        let misbehaviour_height = Height::new(1, 21).unwrap();
        let chain_id_b = ChainId::new("mockgaiaB".to_string(), 1);

        let ctx = MockContext::new(
            ChainId::new("mockgaiaA".to_string(), 1),
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

        let signer = get_dummy_account_id();
        let header1 = {
            let mut tm_block = HostBlock::generate_tm_block(
                chain_id_b.clone(),
                misbehaviour_height.revision_height(),
                Timestamp::now(),
            );
            tm_block.trusted_height = client_height;
            tm_block
        };
        let header2 = {
            let timestamp =
                Timestamp::from_nanoseconds(Timestamp::now().nanoseconds() + 1_000_000_000)
                    .unwrap();
            let mut tm_block = HostBlock::generate_tm_block(
                chain_id_b.clone(),
                misbehaviour_height.revision_height(),
                timestamp,
            );
            tm_block.trusted_height = client_height;
            tm_block
        };

        let msg = MsgSubmitMisbehaviour {
            client_id: client_id.clone(),
            misbehaviour: TmMisbehaviour::new(client_id.clone(), header1.into(), header2.into())
                .unwrap()
                .into(),
            signer,
        };

        let output = dispatch(&ctx, ClientMsg::Misbehaviour(msg));
        match output {
            Ok(HandlerOutput {
                result,
                events: _,
                log,
            }) => {
                assert!(log.is_empty());
                // Check the result
                match result {
                    ClientResult::Misbehaviour(upd_res) => {
                        assert!(upd_res.client_state.is_frozen())
                    }
                    _ => panic!("misbehaviour handler result has incorrect type"),
                }
            }
            Err(err) => {
                panic!("unexpected error: {}", err);
            }
        }
    }

    #[test]
    fn test_misbehaviour_client_events() {
        let client_id = ClientId::default();
        let signer = get_dummy_account_id();

        let timestamp = Timestamp::now();

        let ctx = MockContext::default().with_client(&client_id, Height::new(0, 42).unwrap());
        let height = Height::new(0, 46).unwrap();
        let header = MockHeader::new(height).with_timestamp(timestamp);
        let msg = MsgSubmitMisbehaviour {
            client_id: client_id.clone(),
            misbehaviour: Misbehaviour {
                client_id: client_id.clone(),
                header1: header,
                header2: header,
            }
            .into(),
            signer,
        };

        let output = dispatch(&ctx, ClientMsg::Misbehaviour(msg)).unwrap();
        let misbehaviour_client_event =
            downcast!(output.events.first().unwrap() => IbcEvent::ClientMisbehaviour).unwrap();

        assert_eq!(misbehaviour_client_event.client_id(), &client_id);
        assert_eq!(misbehaviour_client_event.client_type(), &mock_client_type());
    }
}
