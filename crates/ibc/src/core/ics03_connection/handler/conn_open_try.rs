//! Protocol logic specific to processing ICS3 messages of type `MsgConnectionOpenTry`.

use crate::core::ics03_connection::connection::{ConnectionEnd, Counterparty, State};
use crate::core::ics03_connection::context::ConnectionReader;
use crate::core::ics03_connection::error::Error;
use crate::core::ics03_connection::events::Attributes;
use crate::core::ics03_connection::handler::ConnectionResult;
use crate::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
use crate::core::ics24_host::identifier::ConnectionId;
use crate::events::IbcEvent;
use crate::handler::{HandlerOutput, HandlerResult};
use crate::prelude::*;

use super::ConnectionIdState;

pub(crate) fn process(
    ctx: &dyn ConnectionReader,
    msg: MsgConnectionOpenTry,
) -> HandlerResult<ConnectionResult, Error> {
    let mut output = HandlerOutput::builder();

    let connection_id_counter = ctx.connection_counter()?;
    let self_connection_id = ConnectionId::new(connection_id_counter);

    ///////////////////////////////////////////////////////////
    // validate_self_client() verification goes here
    // See [issue](https://github.com/cosmos/ibc-rs/issues/162)
    ///////////////////////////////////////////////////////////

    if msg.consensus_height > ctx.host_current_height() {
        // Fail if the consensus height is too advanced.
        return Err(Error::invalid_consensus_height(
            msg.consensus_height,
            ctx.host_current_height(),
        ));
    }

    let version = ctx.pick_version(
        ctx.get_compatible_versions(),
        msg.counterparty_versions.clone(),
    )?;

    let self_connection_end = ConnectionEnd::new(
        State::TryOpen,
        msg.client_id.clone(),
        msg.counterparty.clone(),
        vec![version],
        msg.delay_period,
    );

    {
        let client_state = ctx.client_state(self_connection_end.client_id())?;
        let consensus_state =
            ctx.client_consensus_state(self_connection_end.client_id(), msg.proofs_height)?;

        {
            let counterparty_connection_id = self_connection_end
                .counterparty()
                .connection_id()
                .ok_or_else(Error::invalid_counterparty)?;
            let counterparty_expected_connection_end = ConnectionEnd::new(
                State::Init,
                msg.counterparty.client_id().clone(),
                Counterparty::new(msg.client_id.clone(), None, ctx.commitment_prefix()),
                msg.counterparty_versions.clone(),
                msg.delay_period,
            );

            client_state
                .verify_connection_state(
                    msg.proofs_height,
                    self_connection_end.counterparty().prefix(),
                    &msg.proof_connection_end,
                    consensus_state.root(),
                    counterparty_connection_id,
                    &counterparty_expected_connection_end,
                )
                .map_err(Error::verify_connection_state)?;
        }

        client_state
            .verify_client_full_state(
                msg.proofs_height,
                self_connection_end.counterparty().prefix(),
                &msg.proof_client_state,
                consensus_state.root(),
                self_connection_end.counterparty().client_id(),
                msg.client_state,
            )
            .map_err(|e| {
                Error::client_state_verification_failure(self_connection_end.client_id().clone(), e)
            })?;

        let expected_consensus_state = ctx.host_consensus_state(msg.consensus_height)?;
        client_state
            .verify_client_consensus_state(
                msg.proofs_height,
                self_connection_end.counterparty().prefix(),
                &msg.proof_consensus_state,
                consensus_state.root(),
                self_connection_end.counterparty().client_id(),
                msg.consensus_height,
                expected_consensus_state.as_ref(),
            )
            .map_err(|e| Error::consensus_state_verification_failure(msg.proofs_height, e))?;
    }

    let result = ConnectionResult {
        connection_id: self_connection_id.clone(),
        connection_end: self_connection_end,
        connection_id_state: ConnectionIdState::Generated,
    };

    let event_attributes = Attributes {
        connection_id: Some(self_connection_id),
        ..Default::default()
    };

    output.emit(IbcEvent::OpenTryConnection(event_attributes.into()));
    output.log("success: conn_open_try verification passed");

    Ok(output.with_result(result))
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use test_log::test;

    use crate::core::ics03_connection::connection::State;
    use crate::core::ics03_connection::handler::{dispatch, ConnectionResult};
    use crate::core::ics03_connection::msgs::conn_open_try::test_util::get_dummy_raw_msg_conn_open_try;
    use crate::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
    use crate::core::ics03_connection::msgs::ConnectionMsg;
    use crate::core::ics24_host::identifier::ChainId;
    use crate::events::IbcEvent;
    use crate::mock::context::MockContext;
    use crate::mock::host::HostType;
    use crate::Height;

    #[test]
    fn conn_open_try_msg_processing() {
        struct Test {
            name: String,
            ctx: MockContext,
            msg: ConnectionMsg,
            want_pass: bool,
        }

        let host_chain_height = Height::new(0, 35).unwrap();
        let max_history_size = 5;
        let context = MockContext::new(
            ChainId::new("mockgaia".to_string(), 0),
            HostType::Mock,
            max_history_size,
            host_chain_height,
        );
        let client_consensus_state_height = 10;

        let msg_conn_try = MsgConnectionOpenTry::try_from(get_dummy_raw_msg_conn_open_try(
            client_consensus_state_height,
            host_chain_height.revision_height(),
        ))
        .unwrap();

        // The proof targets a height that does not exist (i.e., too advanced) on destination chain.
        let msg_height_advanced = MsgConnectionOpenTry::try_from(get_dummy_raw_msg_conn_open_try(
            client_consensus_state_height,
            host_chain_height.increment().revision_height(),
        ))
        .unwrap();
        let pruned_height = host_chain_height
            .sub(max_history_size as u64 + 1)
            .unwrap()
            .revision_height();
        // The consensus proof targets a missing height (pruned) on destination chain.
        let msg_height_old = MsgConnectionOpenTry::try_from(get_dummy_raw_msg_conn_open_try(
            client_consensus_state_height,
            pruned_height,
        ))
        .unwrap();

        // The proofs in this message are created at a height which the client on destination chain does not have.
        let msg_proof_height_missing =
            MsgConnectionOpenTry::try_from(get_dummy_raw_msg_conn_open_try(
                client_consensus_state_height - 1,
                host_chain_height.revision_height(),
            ))
            .unwrap();

        let tests: Vec<Test> = vec![
            Test {
                name: "Processing fails because the height is too advanced".to_string(),
                ctx: context.clone(),
                msg: ConnectionMsg::ConnectionOpenTry(Box::new(msg_height_advanced)),
                want_pass: false,
            },
            Test {
                name: "Processing fails because the height is too old".to_string(),
                ctx: context.clone(),
                msg: ConnectionMsg::ConnectionOpenTry(Box::new(msg_height_old)),
                want_pass: false,
            },
            Test {
                name: "Processing fails because no client exists".to_string(),
                ctx: context.clone(),
                msg: ConnectionMsg::ConnectionOpenTry(Box::new(msg_conn_try.clone())),
                want_pass: false,
            },
            Test {
                name: "Processing fails because the client misses the consensus state targeted by the proof".to_string(),
                ctx: context.clone().with_client(&msg_proof_height_missing.client_id, Height::new(0, client_consensus_state_height).unwrap()),
                msg: ConnectionMsg::ConnectionOpenTry(Box::new(msg_proof_height_missing)),
                want_pass: false,
            },
            Test {
                name: "Good parameters (no previous_connection_id)".to_string(),
                ctx: context.clone().with_client(&msg_conn_try.client_id, Height::new(0, client_consensus_state_height).unwrap()),
                msg: ConnectionMsg::ConnectionOpenTry(Box::new(msg_conn_try.clone())),
                want_pass: true,
            },
            Test {
                name: "Good parameters".to_string(),
                ctx: context.with_client(&msg_conn_try.client_id, Height::new(0, client_consensus_state_height).unwrap()),
                msg: ConnectionMsg::ConnectionOpenTry(Box::new(msg_conn_try)),
                want_pass: true,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let res = dispatch(&test.ctx, test.msg.clone());
            // Additionally check the events and the output objects in the result.
            match res {
                Ok(proto_output) => {
                    assert!(
                        test.want_pass,
                        "conn_open_try: test passed but was supposed to fail for test: {}, \nparams {:?} {:?}",
                        test.name,
                        test.msg.clone(),
                        test.ctx.clone()
                    );

                    assert!(!proto_output.events.is_empty()); // Some events must exist.

                    // The object in the output is a ConnectionEnd, should have TryOpen state.
                    let res: ConnectionResult = proto_output.result;
                    assert_eq!(res.connection_end.state().clone(), State::TryOpen);

                    for e in proto_output.events.iter() {
                        assert!(matches!(e, &IbcEvent::OpenTryConnection(_)));
                    }
                }
                Err(e) => {
                    assert!(
                        !test.want_pass,
                        "conn_open_try: failed for test: {}, \nparams {:?} {:?} error: {:?}",
                        test.name,
                        test.msg,
                        test.ctx.clone(),
                        e,
                    );
                }
            }
        }
    }
}
