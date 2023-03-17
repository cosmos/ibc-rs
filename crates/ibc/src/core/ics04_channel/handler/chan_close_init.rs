//! Protocol logic specific to ICS4 messages of type `MsgChannelCloseInit`.
use crate::core::ics03_connection::connection::State as ConnectionState;
use crate::core::ics04_channel::channel::State;
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::msgs::chan_close_init::MsgChannelCloseInit;
use crate::core::ics24_host::path::ChannelEndPath;

use crate::core::{ContextError, ValidationContext};

pub fn validate<Ctx>(ctx_a: &Ctx, msg: &MsgChannelCloseInit) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let chan_end_path_on_a = ChannelEndPath::new(&msg.port_id_on_a, &msg.chan_id_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    // Validate that the channel end is in a state where it can be closed.
    if chan_end_on_a.state_matches(&State::Closed) {
        return Err(ChannelError::InvalidChannelState {
            channel_id: msg.chan_id_on_a.clone(),
            state: chan_end_on_a.state,
        }
        .into());
    }

    // An OPEN IBC connection running on the local (host) chain should exist.
    if chan_end_on_a.connection_hops().len() != 1 {
        return Err(ChannelError::InvalidConnectionHopsLength {
            expected: 1,
            actual: chan_end_on_a.connection_hops().len(),
        }
        .into());
    }

    let conn_end_on_a = ctx_a.connection_end(&chan_end_on_a.connection_hops()[0])?;

    if !conn_end_on_a.state_matches(&ConnectionState::Open) {
        return Err(ChannelError::ConnectionNotOpen {
            connection_id: chan_end_on_a.connection_hops()[0].clone(),
        }
        .into());
    }

    let client_id_on_a = conn_end_on_a.client_id();
    let client_state_of_b_on_a = ctx_a.client_state(client_id_on_a)?;
    client_state_of_b_on_a.confirm_not_frozen()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::core::ics04_channel::msgs::chan_close_init::test_util::get_dummy_raw_msg_chan_close_init;
    use crate::core::ics04_channel::msgs::chan_close_init::MsgChannelCloseInit;
    use crate::core::ValidationContext;
    use crate::prelude::*;

    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::msgs::test_util::get_dummy_raw_counterparty;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::channel::{
        ChannelEnd, Counterparty, Order, State as ChannelState,
    };
    use crate::core::ics04_channel::Version;
    use crate::core::ics24_host::identifier::{ClientId, ConnectionId};

    use crate::mock::client_state::client_type as mock_client_type;
    use crate::mock::context::MockContext;
    use crate::timestamp::ZERO_DURATION;

    use super::validate;

    #[test]
    fn chan_close_init_event_height() {
        let client_id = ClientId::new(mock_client_type(), 24).unwrap();
        let conn_id = ConnectionId::new(2);

        let conn_end = ConnectionEnd::new(
            ConnectionState::Open,
            client_id.clone(),
            ConnectionCounterparty::try_from(get_dummy_raw_counterparty(Some(0))).unwrap(),
            get_compatible_versions(),
            ZERO_DURATION,
        );

        let msg_chan_close_init =
            MsgChannelCloseInit::try_from(get_dummy_raw_msg_chan_close_init()).unwrap();

        let chan_end = ChannelEnd::new(
            ChannelState::Open,
            Order::default(),
            Counterparty::new(
                msg_chan_close_init.port_id_on_a.clone(),
                Some(msg_chan_close_init.chan_id_on_a.clone()),
            ),
            vec![conn_id.clone()],
            Version::default(),
        );

        let context = {
            let default_context = MockContext::default();
            let client_consensus_state_height = default_context.host_height().unwrap();

            default_context
                .with_client(&client_id, client_consensus_state_height)
                .with_connection(conn_id, conn_end)
                .with_channel(
                    msg_chan_close_init.port_id_on_a.clone(),
                    msg_chan_close_init.chan_id_on_a.clone(),
                    chan_end,
                )
        };

        let res = validate(&context, &msg_chan_close_init);
        assert!(
            res.is_ok(),
            "Validation expected to succeed (happy path). Error: {res:?}"
        );
    }
}
