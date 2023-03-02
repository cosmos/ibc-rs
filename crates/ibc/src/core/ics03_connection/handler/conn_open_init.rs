//! Protocol logic specific to ICS3 messages of type `MsgConnectionOpenInit`.
use crate::prelude::*;

use crate::core::context::ContextError;
use crate::core::ics03_connection::connection::{ConnectionEnd, Counterparty, State};
use crate::core::ics03_connection::error::ConnectionError;
use crate::core::ics03_connection::events::OpenInit;
use crate::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
use crate::core::ics24_host::identifier::ConnectionId;
use crate::core::ics24_host::path::{ClientConnectionPath, ConnectionPath};
use crate::core::{ExecutionContext, ValidationContext};
use crate::events::IbcEvent;

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
        "success: conn_open_init: generated new connection identifier: {conn_id_on_a}"
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
        &ClientConnectionPath::new(&msg.client_id_on_a),
        conn_id_on_a.clone(),
    )?;
    ctx_a.store_connection(&ConnectionPath::new(&conn_id_on_a), conn_end_on_a)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::ics03_connection::connection::State;
    use crate::core::ics03_connection::handler::test_util::{Expect, Fixture};
    use crate::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
    use crate::core::ics03_connection::version::Version;
    use crate::events::IbcEvent;
    use crate::mock::context::MockContext;
    use crate::Height;
    use test_log::test;

    enum Ctx {
        Default,
        WithClient,
    }

    enum Msg {
        Default,
        NoVersion,
        BadVersion,
        WithCounterpartyConnId,
    }

    fn conn_open_init_fixture(
        ctx_variant: Ctx,
        msg_variant: Msg,
    ) -> Fixture<MsgConnectionOpenInit> {
        let msg_default = MsgConnectionOpenInit::new_dummy();
        let msg = match msg_variant {
            Msg::Default => msg_default,
            Msg::NoVersion => msg_default.with_version(None),
            Msg::BadVersion => msg_default.with_version(Some("random identifier 424242")),
            Msg::WithCounterpartyConnId => msg_default.with_counterparty_conn_id(2),
        };

        let ctx_default = MockContext::default();
        let ctx = match ctx_variant {
            Ctx::WithClient => {
                ctx_default.with_client(&msg.client_id_on_a, Height::new(0, 10).unwrap())
            }
            _ => ctx_default,
        };

        Fixture { ctx, msg }
    }

    fn conn_open_init_validate(fxt: &Fixture<MsgConnectionOpenInit>, expect: Expect) {
        let res = validate(&fxt.ctx, fxt.msg.clone());
        let err_msg = fxt.generate_error_msg(&expect, "validation", &res);
        match expect {
            Expect::Failure(_) => {
                assert!(res.is_err(), "{err_msg}")
            }
            Expect::Success => {
                assert!(res.is_ok(), "{err_msg}")
            }
        }
    }

    fn conn_open_init_execute(
        fxt: &mut Fixture<MsgConnectionOpenInit>,
        expect: Expect,
        expected_version: Vec<Version>,
    ) {
        let res = execute(&mut fxt.ctx, fxt.msg.clone());
        let err_msg = fxt.generate_error_msg(&expect, "execution", &res);
        match expect {
            Expect::Failure(_) => {
                assert!(res.is_err(), "{err_msg}")
            }
            Expect::Success => {
                assert!(res.is_ok(), "{err_msg}");
                assert_eq!(fxt.ctx.events.len(), 1);

                let event = fxt.ctx.events.first().unwrap();
                assert!(matches!(event, &IbcEvent::OpenInitConnection(_)));

                let conn_open_init_event = match event {
                    IbcEvent::OpenInitConnection(e) => e,
                    _ => unreachable!(),
                };
                let conn_end = <MockContext as ValidationContext>::connection_end(
                    &fxt.ctx,
                    conn_open_init_event.connection_id(),
                )
                .unwrap();
                assert_eq!(conn_end.state().clone(), State::Init);
                assert_eq!(conn_end.versions(), expected_version);
            }
        }
    }

    #[test]
    fn conn_open_init_healthy() {
        let mut fxt = conn_open_init_fixture(Ctx::WithClient, Msg::Default);
        conn_open_init_validate(&fxt, Expect::Success);
        let expected_version = vec![fxt.msg.version.clone().unwrap()];
        conn_open_init_execute(&mut fxt, Expect::Success, expected_version);
    }

    #[test]
    fn conn_open_init_no_context() {
        let fxt = conn_open_init_fixture(Ctx::Default, Msg::Default);
        conn_open_init_validate(&fxt, Expect::Failure(None));
    }

    #[test]
    fn conn_open_init_no_version() {
        let mut fxt = conn_open_init_fixture(Ctx::WithClient, Msg::NoVersion);
        conn_open_init_validate(&fxt, Expect::Success);
        let expected_version = ValidationContext::get_compatible_versions(&fxt.ctx.clone());
        conn_open_init_execute(&mut fxt, Expect::Success, expected_version);
    }
    #[test]
    fn conn_open_init_incompatible_version() {
        let fxt = conn_open_init_fixture(Ctx::WithClient, Msg::BadVersion);
        conn_open_init_validate(&fxt, Expect::Failure(None));
    }

    #[test]
    fn conn_open_init_with_counterparty_conn_id() {
        let mut fxt = conn_open_init_fixture(Ctx::WithClient, Msg::WithCounterpartyConnId);
        conn_open_init_validate(&fxt, Expect::Success);
        let expected_version = vec![fxt.msg.version.clone().unwrap()];
        conn_open_init_execute(&mut fxt, Expect::Success, expected_version);
    }
}
