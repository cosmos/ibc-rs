//! Protocol logic specific to ICS3 messages of type `MsgConnectionOpenInit`.
use crate::prelude::*;

use crate::core::ics03_connection::connection::{ConnectionEnd, Counterparty, State};
use crate::core::ics03_connection::context::ConnectionReader;
use crate::core::ics03_connection::error::ConnectionError;
use crate::core::ics03_connection::events::OpenInit;
use crate::core::ics03_connection::handler::ConnectionResult;
use crate::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
use crate::core::ics24_host::identifier::ConnectionId;
use crate::events::IbcEvent;
use crate::handler::{HandlerOutput, HandlerResult};

#[cfg(feature = "val_exec_ctx")]
use crate::core::context::ContextError;
#[cfg(feature = "val_exec_ctx")]
use crate::core::ics24_host::path::{ClientConnectionsPath, ConnectionsPath};
#[cfg(feature = "val_exec_ctx")]
use crate::core::{ExecutionContext, ValidationContext};

use super::ConnectionIdState;

#[cfg(feature = "val_exec_ctx")]
pub(crate) fn validate<Ctx>(ctx_a: &Ctx, msg: MsgConnectionOpenInit) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    // An IBC client running on the local (host) chain should exist.
    ctx_a.client_state(&msg.client_id_on_a)?;

    if let Some(version) = msg.version {
        if !ctx_a.get_compatible_versions().contains(&version) {
            return Err(ConnectionError::VersionNotSupported { version }.into());
        }
    }

    Ok(())
}

#[cfg(feature = "val_exec_ctx")]
pub(crate) fn execute<Ctx>(ctx_a: &mut Ctx, msg: MsgConnectionOpenInit) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    let versions = match msg.version {
        Some(version) => {
            if ctx_a.get_compatible_versions().contains(&version) {
                Ok(vec![version])
            } else {
                Err(ConnectionError::VersionNotSupported { version })
            }
        }
        None => Ok(ctx_a.get_compatible_versions()),
    }?;

    let conn_end_on_a = ConnectionEnd::new(
        State::Init,
        msg.client_id_on_a.clone(),
        Counterparty::new(
            msg.counterparty.client_id().clone(),
            None,
            msg.counterparty.prefix().clone(),
        ),
        versions,
        msg.delay_period,
    );

    // Construct the identifier for the new connection.
    let conn_id_on_a = ConnectionId::new(ctx_a.connection_counter()?);

    ctx_a.log_message(format!(
        "success: conn_open_init: generated new connection identifier: {}",
        conn_id_on_a
    ));

    {
        let client_id_on_b = msg.counterparty.client_id().clone();

        ctx_a.emit_ibc_event(IbcEvent::OpenInitConnection(OpenInit::new(
            conn_id_on_a.clone(),
            msg.client_id_on_a.clone(),
            client_id_on_b,
        )));
    }

    ctx_a.increase_connection_counter();
    ctx_a.store_connection_to_client(
        ClientConnectionsPath(msg.client_id_on_a),
        conn_id_on_a.clone(),
    )?;
    ctx_a.store_connection(ConnectionsPath(conn_id_on_a), conn_end_on_a)?;

    Ok(())
}

/// Per our convention, this message is processed on chain A.
pub(crate) fn process(
    ctx_a: &dyn ConnectionReader,
    msg: MsgConnectionOpenInit,
) -> HandlerResult<ConnectionResult, ConnectionError> {
    let mut output = HandlerOutput::builder();

    // An IBC client running on the local (host) chain should exist.
    ctx_a.client_state(&msg.client_id_on_a)?;

    let versions = match msg.version {
        Some(version) => {
            if ctx_a.get_compatible_versions().contains(&version) {
                Ok(vec![version])
            } else {
                Err(ConnectionError::VersionNotSupported { version })
            }
        }
        None => Ok(ctx_a.get_compatible_versions()),
    }?;

    let conn_end_on_a = ConnectionEnd::new(
        State::Init,
        msg.client_id_on_a.clone(),
        Counterparty::new(
            msg.counterparty.client_id().clone(),
            None,
            msg.counterparty.prefix().clone(),
        ),
        versions,
        msg.delay_period,
    );

    // Construct the identifier for the new connection.
    let conn_id_on_a = ConnectionId::new(ctx_a.connection_counter()?);

    let result = ConnectionResult {
        connection_id: conn_id_on_a.clone(),
        connection_end: conn_end_on_a,
        connection_id_state: ConnectionIdState::Generated,
    };

    output.log(format!(
        "success: conn_open_init: generated new connection identifier: {conn_id_on_a}",
    ));

    {
        let client_id_on_b = msg.counterparty.client_id().clone();

        output.emit(IbcEvent::OpenInitConnection(OpenInit::new(
            conn_id_on_a,
            msg.client_id_on_a,
            client_id_on_b,
        )));
    }

    Ok(output.with_result(result))
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use test_log::test;

    use crate::core::ics03_connection::connection::State;
    use crate::core::ics03_connection::context::ConnectionReader;
    use crate::core::ics03_connection::handler::{dispatch, ConnectionResult};
    use crate::core::ics03_connection::msgs::conn_open_init::test_util::get_dummy_raw_msg_conn_open_init;
    use crate::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
    use crate::core::ics03_connection::msgs::ConnectionMsg;
    use crate::core::ics03_connection::version::Version;
    use crate::events::IbcEvent;
    use crate::mock::context::MockContext;
    use crate::Height;

    #[cfg(feature = "val_exec_ctx")]
    use crate::core::ics26_routing::msgs::MsgEnvelope;
    #[cfg(feature = "val_exec_ctx")]
    use crate::core::ValidationContext;

    use ibc_proto::ibc::core::connection::v1::Version as RawVersion;

    #[test]
    fn conn_open_init_msg_processing() {
        struct Test {
            name: String,
            ctx: MockContext,
            msg: ConnectionMsg,
            expected_versions: Vec<Version>,
            want_pass: bool,
        }

        let msg_conn_init_default =
            MsgConnectionOpenInit::try_from(get_dummy_raw_msg_conn_open_init()).unwrap();
        let msg_conn_init_no_version = MsgConnectionOpenInit {
            version: None,
            ..msg_conn_init_default.clone()
        };
        let msg_conn_init_bad_version = MsgConnectionOpenInit {
            version: Version::try_from(RawVersion {
                identifier: "random identifier 424242".to_string(),
                features: vec![],
            })
            .unwrap()
            .into(),
            ..msg_conn_init_default.clone()
        };
        let default_context = MockContext::default();
        let good_context = default_context.clone().with_client(
            &msg_conn_init_default.client_id_on_a,
            Height::new(0, 10).unwrap(),
        );

        let tests: Vec<Test> = vec![
            Test {
                name: "Processing fails because no client exists in the context".to_string(),
                ctx: default_context,
                msg: ConnectionMsg::Init(msg_conn_init_default.clone()),
                expected_versions: vec![msg_conn_init_default.version.clone().unwrap()],
                want_pass: false,
            },
            Test {
                name: "Incompatible version in MsgConnectionOpenInit msg".to_string(),
                ctx: good_context.clone(),
                msg: ConnectionMsg::Init(msg_conn_init_bad_version),
                expected_versions: vec![],
                want_pass: false,
            },
            Test {
                name: "No version in MsgConnectionOpenInit msg".to_string(),
                ctx: good_context.clone(),
                msg: ConnectionMsg::Init(msg_conn_init_no_version),
                expected_versions: ConnectionReader::get_compatible_versions(&good_context),
                want_pass: true,
            },
            Test {
                name: "Good parameters".to_string(),
                ctx: good_context,
                msg: ConnectionMsg::Init(msg_conn_init_default.clone()),
                expected_versions: vec![msg_conn_init_default.version.unwrap()],
                want_pass: true,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            #[cfg(feature = "val_exec_ctx")]
            {
                let res = ValidationContext::validate(
                    &test.ctx,
                    MsgEnvelope::ConnectionMsg(test.msg.clone()),
                );

                match res {
                    Ok(_) => {
                        assert!(
                        test.want_pass,
                        "conn_open_init: test passed but was supposed to fail for test: {}, \nparams {:?} {:?}",
                        test.name,
                        test.msg.clone(),
                        test.ctx.clone()
                    )
                    }
                    Err(e) => {
                        assert!(
                            !test.want_pass,
                            "conn_open_init: did not pass test: {}, \nparams {:?} {:?} error: {:?}",
                            test.name,
                            test.msg,
                            test.ctx.clone(),
                            e,
                        );
                    }
                }
            }
            let res = dispatch(&test.ctx, test.msg.clone());
            // Additionally check the events and the output objects in the result.
            match res {
                Ok(proto_output) => {
                    assert!(!proto_output.events.is_empty()); // Some events must exist.

                    // The object in the output is a ConnectionEnd, should have init state.
                    let res: ConnectionResult = proto_output.result;
                    assert_eq!(res.connection_end.state().clone(), State::Init);

                    for e in proto_output.events.iter() {
                        assert!(matches!(e, &IbcEvent::OpenInitConnection(_)));
                    }

                    assert_eq!(res.connection_end.versions(), test.expected_versions);

                    // This needs to be last
                    assert!(
                        test.want_pass,
                        "conn_open_init: test passed but was supposed to fail for test: {}, \nparams {:?} {:?}",
                        test.name,
                        test.msg.clone(),
                        test.ctx.clone()
                    );
                }
                Err(e) => {
                    assert!(
                        !test.want_pass,
                        "conn_open_init: did not pass test: {}, \nparams {:?} {:?} error: {:?}",
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
