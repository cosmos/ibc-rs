//! Protocol logic specific to ICS4 messages of type `MsgChannelOpenAck`.
use crate::core::ics03_connection::connection::State as ConnectionState;
use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, State};
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::msgs::chan_open_ack::MsgChannelOpenAck;
use crate::core::ics24_host::path::{ChannelEndPath, ClientConsensusStatePath};
use crate::prelude::*;

use crate::core::{ContextError, ValidationContext};

pub fn validate<Ctx>(ctx_a: &Ctx, msg: &MsgChannelOpenAck) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let chan_end_path_on_a = ChannelEndPath::new(&msg.port_id_on_a, &msg.chan_id_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    // Validate that the channel end is in a state where it can be ack.
    if !chan_end_on_a.state_matches(&State::Init) {
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

    // Verify proofs
    {
        let client_id_on_a = conn_end_on_a.client_id();
        let client_state_of_b_on_a = ctx_a.client_state(client_id_on_a)?;
        let client_cons_state_path_on_a =
            ClientConsensusStatePath::new(client_id_on_a, &msg.proof_height_on_b);
        let consensus_state_of_b_on_a = ctx_a.consensus_state(&client_cons_state_path_on_a)?;
        let prefix_on_b = conn_end_on_a.counterparty().prefix();
        let port_id_on_b = &chan_end_on_a.counterparty().port_id;
        let conn_id_on_b = conn_end_on_a.counterparty().connection_id().ok_or(
            ChannelError::UndefinedConnectionCounterparty {
                connection_id: chan_end_on_a.connection_hops()[0].clone(),
            },
        )?;

        // The client must not be frozen.
        if client_state_of_b_on_a.is_frozen() {
            return Err(ChannelError::FrozenClient {
                client_id: client_id_on_a.clone(),
            }
            .into());
        }

        let expected_chan_end_on_b = ChannelEnd::new(
            State::TryOpen,
            // Note: Both ends of a channel must have the same ordering, so it's
            // fine to use A's ordering here
            *chan_end_on_a.ordering(),
            Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_a.clone())),
            vec![conn_id_on_b.clone()],
            msg.version_on_b.clone(),
        );
        let chan_end_path_on_b = ChannelEndPath::new(port_id_on_b, &msg.chan_id_on_b);

        // Verify the proof for the channel state against the expected channel end.
        // A counterparty channel id of None in not possible, and is checked by validate_basic in msg.
        client_state_of_b_on_a
            .verify_channel_state(
                msg.proof_height_on_b,
                prefix_on_b,
                &msg.proof_chan_end_on_b,
                consensus_state_of_b_on_a.root(),
                &chan_end_path_on_b,
                &expected_chan_end_on_b,
            )
            .map_err(ChannelError::VerifyChannelFailed)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::core::ics03_connection::msgs::test_util::get_dummy_raw_counterparty;
    use crate::core::ics04_channel::channel::Order;
    use crate::core::ics04_channel::handler::chan_open_ack::validate;
    use crate::core::ics24_host::identifier::ClientId;
    use crate::prelude::*;
    use crate::timestamp::ZERO_DURATION;
    use rstest::*;
    use test_log::test;

    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, State};
    use crate::core::ics04_channel::msgs::chan_open_ack::test_util::get_dummy_raw_msg_chan_open_ack;
    use crate::core::ics04_channel::msgs::chan_open_ack::MsgChannelOpenAck;
    use crate::core::ics24_host::identifier::ConnectionId;
    use crate::mock::client_state::client_type as mock_client_type;
    use crate::mock::context::MockContext;
    use crate::Height;

    pub struct Fixture {
        pub context: MockContext,
        pub msg: MsgChannelOpenAck,
        pub client_id_on_a: ClientId,
        pub conn_id_on_a: ConnectionId,
        pub conn_end_on_a: ConnectionEnd,
        pub chan_end_on_a: ChannelEnd,
        pub proof_height: u64,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let proof_height = 10;
        let context = MockContext::default();

        let client_id_on_a = ClientId::new(mock_client_type(), 45).unwrap();
        let conn_id_on_a = ConnectionId::new(2);
        let conn_end_on_a = ConnectionEnd::new(
            ConnectionState::Open,
            client_id_on_a.clone(),
            ConnectionCounterparty::try_from(get_dummy_raw_counterparty(Some(0))).unwrap(),
            get_compatible_versions(),
            ZERO_DURATION,
        );

        let msg =
            MsgChannelOpenAck::try_from(get_dummy_raw_msg_chan_open_ack(proof_height)).unwrap();

        let chan_end_on_a = ChannelEnd::new(
            State::Init,
            Order::Unordered,
            Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_b.clone())),
            vec![conn_id_on_a.clone()],
            msg.version_on_b.clone(),
        );

        Fixture {
            context,
            msg,
            client_id_on_a,
            conn_id_on_a,
            conn_end_on_a,
            chan_end_on_a,
            proof_height,
        }
    }

    #[rstest]
    fn chan_open_ack_fail_no_channel(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            client_id_on_a,
            conn_id_on_a,
            conn_end_on_a,
            proof_height,
            ..
        } = fixture;
        let context = context
            .with_client(&client_id_on_a, Height::new(0, proof_height).unwrap())
            .with_connection(conn_id_on_a, conn_end_on_a);

        let res = validate(&context, &msg);

        assert!(
            res.is_err(),
            "Validation fails because no channel exists in the context"
        )
    }

    #[rstest]
    fn chan_open_ack_fail_channel_wrong_state(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            client_id_on_a,
            conn_id_on_a,
            conn_end_on_a,
            proof_height,
            ..
        } = fixture;

        let wrong_chan_end = ChannelEnd::new(
            State::Open,
            Order::Unordered,
            Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_b.clone())),
            vec![conn_id_on_a.clone()],
            msg.version_on_b.clone(),
        );
        let context = context
            .with_client(&client_id_on_a, Height::new(0, proof_height).unwrap())
            .with_connection(conn_id_on_a, conn_end_on_a)
            .with_channel(
                msg.port_id_on_a.clone(),
                msg.chan_id_on_a.clone(),
                wrong_chan_end,
            );

        let res = validate(&context, &msg);

        assert!(
            res.is_err(),
            "Validation fails because channel is in the wrong state"
        )
    }

    #[rstest]
    fn chan_open_ack_fail_no_connection(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            client_id_on_a,
            chan_end_on_a,
            proof_height,
            ..
        } = fixture;

        let context = context
            .with_client(&client_id_on_a, Height::new(0, proof_height).unwrap())
            .with_channel(
                msg.port_id_on_a.clone(),
                msg.chan_id_on_a.clone(),
                chan_end_on_a,
            );

        let res = validate(&context, &msg);

        assert!(
            res.is_err(),
            "Validation fails because no connection exists in the context"
        )
    }

    #[rstest]
    fn chan_open_ack_happy_path(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            client_id_on_a,
            conn_id_on_a,
            conn_end_on_a,
            chan_end_on_a,
            proof_height,
            ..
        } = fixture;

        let context = context
            .with_client(&client_id_on_a, Height::new(0, proof_height).unwrap())
            .with_connection(conn_id_on_a, conn_end_on_a)
            .with_channel(
                msg.port_id_on_a.clone(),
                msg.chan_id_on_a.clone(),
                chan_end_on_a,
            );

        let res = validate(&context, &msg);

        assert!(res.is_ok(), "Validation happy path")
    }
}
