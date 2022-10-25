//! Protocol logic specific to processing ICS3 messages of type `MsgConnectionOpenConfirm`.

use crate::core::ics03_connection::connection::{ConnectionEnd, Counterparty, State};
use crate::core::ics03_connection::context::ConnectionReader;
use crate::core::ics03_connection::error::Error;
use crate::core::ics03_connection::events::Attributes;
use crate::core::ics03_connection::handler::{ConnectionIdState, ConnectionResult};
use crate::core::ics03_connection::msgs::conn_open_confirm::MsgConnectionOpenConfirm;
use crate::events::IbcEvent;
use crate::handler::{HandlerOutput, HandlerResult};
use crate::prelude::*;

/// Per our convention, this message is processed on chain B.
pub(crate) fn process(
    ctx_b: &dyn ConnectionReader,
    msg: MsgConnectionOpenConfirm,
) -> HandlerResult<ConnectionResult, Error> {
    let mut output = HandlerOutput::builder();

    let conn_end_on_b = ctx_b.connection_end(&msg.conn_id_on_b)?;
    if !conn_end_on_b.state_matches(&State::TryOpen) {
        return Err(Error::connection_mismatch(msg.conn_id_on_b));
    }

    // Verify proofs
    {
        let client_state_of_a_on_b = ctx_b.client_state(conn_end_on_b.client_id())?;
        let consensus_state_of_a_on_b =
            ctx_b.client_consensus_state(conn_end_on_b.client_id(), msg.proof_height_on_a)?;

        let client_id_on_a = conn_end_on_b.counterparty().client_id();
        let client_id_on_b = conn_end_on_b.client_id();
        let conn_id_on_a = conn_end_on_b
            .counterparty()
            .connection_id()
            .ok_or_else(Error::invalid_counterparty)?;
        let prefix_on_a = conn_end_on_b.counterparty().prefix();
        let prefix_on_b = ctx_b.commitment_prefix();

        let expected_conn_end_on_a = ConnectionEnd::new(
            State::Open,
            client_id_on_a.clone(),
            Counterparty::new(
                client_id_on_b.clone(),
                Some(msg.conn_id_on_b.clone()),
                prefix_on_b,
            ),
            conn_end_on_b.versions().to_vec(),
            conn_end_on_b.delay_period(),
        );

        client_state_of_a_on_b
            .verify_connection_state(
                msg.proof_height_on_a,
                prefix_on_a,
                &msg.proof_conn_end_on_a,
                consensus_state_of_a_on_b.root(),
                conn_id_on_a,
                &expected_conn_end_on_a,
            )
            .map_err(Error::verify_connection_state)?;
    }

    // Success
    let result = {
        let new_conn_end_on_b = {
            let mut new_conn_end_on_b = conn_end_on_b;

            new_conn_end_on_b.set_state(State::Open);
            new_conn_end_on_b
        };

        ConnectionResult {
            connection_id: msg.conn_id_on_b,
            connection_id_state: ConnectionIdState::Reused,
            connection_end: new_conn_end_on_b,
        }
    };

    let event_attributes = Attributes {
        connection_id: Some(result.connection_id.clone()),
        ..Default::default()
    };

    output.emit(IbcEvent::OpenConfirmConnection(event_attributes.into()));
    output.log("success: conn_open_confirm verification passed");

    Ok(output.with_result(result))
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use core::str::FromStr;
    use test_log::test;

    use crate::core::ics03_connection::connection::{ConnectionEnd, Counterparty, State};
    use crate::core::ics03_connection::context::ConnectionReader;
    use crate::core::ics03_connection::handler::{dispatch, ConnectionResult};
    use crate::core::ics03_connection::msgs::conn_open_confirm::test_util::get_dummy_raw_msg_conn_open_confirm;
    use crate::core::ics03_connection::msgs::conn_open_confirm::MsgConnectionOpenConfirm;
    use crate::core::ics03_connection::msgs::ConnectionMsg;
    use crate::core::ics23_commitment::commitment::CommitmentPrefix;
    use crate::core::ics24_host::identifier::ClientId;
    use crate::events::IbcEvent;
    use crate::mock::context::MockContext;
    use crate::timestamp::ZERO_DURATION;
    use crate::Height;

    #[test]
    fn conn_open_confirm_msg_processing() {
        struct Test {
            name: String,
            ctx: MockContext,
            msg: ConnectionMsg,
            want_pass: bool,
        }

        let client_id = ClientId::from_str("mock_clientid").unwrap();
        let msg_confirm =
            MsgConnectionOpenConfirm::try_from(get_dummy_raw_msg_conn_open_confirm()).unwrap();
        let counterparty = Counterparty::new(
            client_id.clone(),
            Some(msg_confirm.conn_id_on_b.clone()),
            CommitmentPrefix::try_from(b"ibc".to_vec()).unwrap(),
        );

        let context = MockContext::default();

        let incorrect_conn_end_state = ConnectionEnd::new(
            State::Init,
            client_id.clone(),
            counterparty,
            context.get_compatible_versions(),
            ZERO_DURATION,
        );

        let mut correct_conn_end = incorrect_conn_end_state.clone();
        correct_conn_end.set_state(State::TryOpen);

        let tests: Vec<Test> = vec![
            Test {
                name: "Processing fails due to missing connection in context".to_string(),
                ctx: context.clone(),
                msg: ConnectionMsg::ConnectionOpenConfirm(msg_confirm.clone()),
                want_pass: false,
            },
            Test {
                name: "Processing fails due to connections mismatch (incorrect state)".to_string(),
                ctx: context
                    .clone()
                    .with_client(&client_id, Height::new(0, 10).unwrap())
                    .with_connection(msg_confirm.conn_id_on_b.clone(), incorrect_conn_end_state),
                msg: ConnectionMsg::ConnectionOpenConfirm(msg_confirm.clone()),
                want_pass: false,
            },
            Test {
                name: "Processing successful".to_string(),
                ctx: context
                    .with_client(&client_id, Height::new(0, 10).unwrap())
                    .with_connection(msg_confirm.conn_id_on_b.clone(), correct_conn_end),
                msg: ConnectionMsg::ConnectionOpenConfirm(msg_confirm),
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
                        "conn_open_confirm: test passed but was supposed to fail for: {}, \nparams {:?} {:?}",
                        test.name,
                        test.msg.clone(),
                        test.ctx.clone()
                    );

                    assert!(!proto_output.events.is_empty()); // Some events must exist.

                    // The object in the output is a ConnectionEnd, should have OPEN state.
                    let res: ConnectionResult = proto_output.result;
                    assert_eq!(res.connection_end.state().clone(), State::Open);

                    for e in proto_output.events.iter() {
                        assert!(matches!(e, &IbcEvent::OpenConfirmConnection(_)));
                    }
                }
                Err(e) => {
                    assert!(
                        !test.want_pass,
                        "conn_open_confirm: failed for test: {}, \nparams {:?} {:?} error: {:?}",
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
