//! Protocol logic specific to ICS4 messages of type `MsgChannelOpenInit`.

use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::msgs::chan_open_init::MsgChannelOpenInit;
use crate::prelude::*;

use crate::core::{ContextError, ValidationContext};

pub fn validate<Ctx>(ctx_a: &Ctx, msg: &MsgChannelOpenInit) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    if msg.connection_hops_on_a.len() != 1 {
        return Err(ChannelError::InvalidConnectionHopsLength {
            expected: 1,
            actual: msg.connection_hops_on_a.len(),
        }
        .into());
    }

    // An IBC connection running on the local (host) chain should exist.
    let conn_end_on_a = ctx_a.connection_end(&msg.connection_hops_on_a[0])?;

    let conn_version = match conn_end_on_a.versions() {
        [version] => version,
        _ => return Err(ChannelError::InvalidVersionLengthConnection.into()),
    };

    let channel_feature = msg.ordering.to_string();
    if !conn_version.is_supported_feature(channel_feature) {
        return Err(ChannelError::ChannelFeatureNotSuportedByConnection.into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::core::ics04_channel::handler::chan_open_init::validate;
    use crate::prelude::*;
    use rstest::*;

    use test_log::test;

    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::msgs::chan_open_init::test_util::get_dummy_raw_msg_chan_open_init;
    use crate::core::ics04_channel::msgs::chan_open_init::MsgChannelOpenInit;
    use crate::core::ics24_host::identifier::ConnectionId;
    use crate::mock::context::MockContext;

    pub struct Fixture {
        pub context: MockContext,
        pub msg: MsgChannelOpenInit,
        pub conn_end_on_a: ConnectionEnd,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let msg = MsgChannelOpenInit::try_from(get_dummy_raw_msg_chan_open_init(None)).unwrap();

        let context = MockContext::default();

        let msg_conn_init = MsgConnectionOpenInit::new_dummy();

        let conn_end_on_a = ConnectionEnd::new(
            ConnectionState::Init,
            msg_conn_init.client_id_on_a.clone(),
            msg_conn_init.counterparty.clone(),
            get_compatible_versions(),
            msg_conn_init.delay_period,
        );

        Fixture {
            context,
            msg,
            conn_end_on_a,
        }
    }

    #[rstest]
    fn chan_open_init_fail_no_connection(fixture: Fixture) {
        let Fixture { context, msg, .. } = fixture;

        let res = validate(&context, &msg);

        assert!(
            res.is_err(),
            "Validation fails because no connection exists in the context"
        )
    }

    #[rstest]
    fn chan_open_init_success_happy_path(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            conn_end_on_a,
        } = fixture;

        let context = context.with_connection(ConnectionId::default(), conn_end_on_a);

        let res = validate(&context, &msg);

        assert!(res.is_ok(), "Validation succeeds; good parameters")
    }

    #[rstest]
    fn chan_open_init_success_counterparty_chan_id_set(fixture: Fixture) {
        let Fixture {
            context,
            conn_end_on_a,
            ..
        } = fixture;

        let context = context.with_connection(ConnectionId::default(), conn_end_on_a);
        let msg = MsgChannelOpenInit::try_from(get_dummy_raw_msg_chan_open_init(Some(0))).unwrap();

        let res = validate(&context, &msg);

        assert!(
            res.is_ok(),
            "Validation succeeds even if counterparty channel id is set by relayer"
        )
    }
}
