//! Protocol logic specific to ICS4 messages of type `MsgChannelOpenInit`.

use crate::core::handler::{ExecutionHandler, ValidationHandler};
use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, State};
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::events::OpenInit;
use crate::core::ics04_channel::msgs::chan_open_init::MsgChannelOpenInit;
use crate::core::ics04_channel::msgs::ChannelMsg;
use crate::core::ics24_host::identifier::ChannelId;
use crate::core::ics24_host::path::{ChannelEndPath, SeqAckPath, SeqRecvPath, SeqSendPath};
use crate::events::IbcEvent;
use crate::prelude::*;

use crate::core::{ContextError, KeeperContext, ReaderContext};

impl<Ctx> ValidationHandler<MsgChannelOpenInit> for Ctx
where
    Ctx: ReaderContext,
{
    fn validate(&self, msg: &MsgChannelOpenInit) -> Result<(), ContextError>
    where
        Ctx: ReaderContext,
    {
        if msg.connection_hops_on_a.len() != 1 {
            return Err(ChannelError::InvalidConnectionHopsLength {
                expected: 1,
                actual: msg.connection_hops_on_a.len(),
            }
            .into());
        }

        // An IBC connection running on the local (host) chain should exist.
        let conn_end_on_a = self.connection_end(&msg.connection_hops_on_a[0])?;
        let conn_version = match conn_end_on_a.versions() {
            [version] => version,
            _ => return Err(ChannelError::InvalidVersionLengthConnection.into()),
        };

        let channel_feature = msg.ordering.to_string();
        if !conn_version.is_supported_feature(channel_feature) {
            return Err(ChannelError::ChannelFeatureNotSuportedByConnection.into());
        }

        let chan_id_on_a = ChannelId::new(self.channel_counter()?);
        let module = {
            let module_id = self
                .lookup_module_channel(&ChannelMsg::OpenInit(msg.clone()))
                .map_err(ContextError::from)?;
            self.get_route(&module_id)
                .ok_or(ChannelError::RouteNotFound)?
        };
        module.on_chan_open_init_validate(
            msg.ordering,
            &msg.connection_hops_on_a,
            &msg.port_id_on_a,
            &chan_id_on_a,
            &Counterparty::new(msg.port_id_on_b.clone(), None),
            &msg.version_proposal,
        )?;

        Ok(())
    }
}

impl<Ctx> ExecutionHandler<MsgChannelOpenInit> for Ctx
where
    Ctx: KeeperContext,
{
    fn execute(&mut self, msg: &MsgChannelOpenInit) -> Result<(), ContextError> {
        let chan_id_on_a = ChannelId::new(self.channel_counter()?);
        let module = {
            let module_id = self
                .lookup_module_channel(&ChannelMsg::OpenInit(msg.clone()))
                .map_err(ContextError::from)?;
            self.get_route_mut(&module_id)
                .ok_or(ChannelError::RouteNotFound)?
        };
        let (extras, version) = module.on_chan_open_init_execute(
            msg.ordering,
            &msg.connection_hops_on_a,
            &msg.port_id_on_a,
            &chan_id_on_a,
            &Counterparty::new(msg.port_id_on_b.clone(), None),
            &msg.version_proposal,
        )?;

        let conn_id_on_a = msg.connection_hops_on_a[0].clone();

        // state changes
        {
            let chan_end_on_a = ChannelEnd::new(
                State::Init,
                msg.ordering,
                Counterparty::new(msg.port_id_on_b.clone(), None),
                msg.connection_hops_on_a.clone(),
                msg.version_proposal.clone(),
            );
            let chan_end_path_on_a = ChannelEndPath::new(&msg.port_id_on_a, &chan_id_on_a);
            self.store_channel(&chan_end_path_on_a, chan_end_on_a)?;

            self.increase_channel_counter();

            // Initialize send, recv, and ack sequence numbers.
            let seq_send_path = SeqSendPath::new(&msg.port_id_on_a, &chan_id_on_a);
            self.store_next_sequence_send(&seq_send_path, 1.into())?;

            let seq_recv_path = SeqRecvPath::new(&msg.port_id_on_a, &chan_id_on_a);
            self.store_next_sequence_recv(&seq_recv_path, 1.into())?;

            let seq_ack_path = SeqAckPath::new(&msg.port_id_on_a, &chan_id_on_a);
            self.store_next_sequence_ack(&seq_ack_path, 1.into())?;
        }

        // emit events and logs
        {
            self.log_message(format!(
                "success: channel open init with channel identifier: {chan_id_on_a}"
            ));
            let core_event = IbcEvent::OpenInitChannel(OpenInit::new(
                msg.port_id_on_a.clone(),
                chan_id_on_a.clone(),
                msg.port_id_on_b.clone(),
                conn_id_on_a,
                version,
            ));
            self.emit_ibc_event(core_event);

            for module_event in extras.events {
                self.emit_ibc_event(IbcEvent::AppModule(module_event));
            }

            for log_message in extras.log {
                self.log_message(log_message);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::core::handler::ExecutionHandler;
    use crate::core::handler::ValidationHandler;
    use crate::events::IbcEvent;
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

        let res = context.validate(&msg);

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

        let res = context.validate(&msg);

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

        let res = context.validate(&msg);

        assert!(
            res.is_ok(),
            "Validation succeeds even if counterparty channel id is set by relayer"
        )
    }

    #[rstest]
    fn chan_open_init_execute_events(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            conn_end_on_a,
        } = fixture;

        let mut context = context.with_connection(ConnectionId::default(), conn_end_on_a);

        let res = context.execute(&msg);

        assert!(res.is_ok(), "Execution succeeds; good parameters");

        assert_eq!(context.events.len(), 1);
        assert!(matches!(
            context.events.first().unwrap(),
            &IbcEvent::OpenInitChannel(_)
        ));
    }
}
