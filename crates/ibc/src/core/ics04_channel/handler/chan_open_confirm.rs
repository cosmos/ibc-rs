//! Protocol logic specific to ICS4 messages of type `MsgChannelOpenConfirm`.
use crate::core::ics03_connection::connection::State as ConnectionState;
use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, State};
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::msgs::chan_open_confirm::MsgChannelOpenConfirm;
use crate::core::ics24_host::path::{ChannelEndPath, ClientConsensusStatePath};
use crate::prelude::*;

use crate::core::{ContextError, ValidationContext};

pub fn validate<Ctx>(ctx_b: &Ctx, msg: &MsgChannelOpenConfirm) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    // Unwrap the old channel end and validate it against the message.
    let chan_end_path_on_b = ChannelEndPath::new(&msg.port_id_on_b, &msg.chan_id_on_b);
    let chan_end_on_b = ctx_b.channel_end(&chan_end_path_on_b)?;

    // Validate that the channel end is in a state where it can be confirmed.
    if !chan_end_on_b.state_matches(&State::TryOpen) {
        return Err(ChannelError::InvalidChannelState {
            channel_id: msg.chan_id_on_b.clone(),
            state: chan_end_on_b.state,
        }
        .into());
    }

    // An OPEN IBC connection running on the local (host) chain should exist.
    if chan_end_on_b.connection_hops().len() != 1 {
        return Err(ChannelError::InvalidConnectionHopsLength {
            expected: 1,
            actual: chan_end_on_b.connection_hops().len(),
        }
        .into());
    }

    let conn_end_on_b = ctx_b.connection_end(&chan_end_on_b.connection_hops()[0])?;

    if !conn_end_on_b.state_matches(&ConnectionState::Open) {
        return Err(ChannelError::ConnectionNotOpen {
            connection_id: chan_end_on_b.connection_hops()[0].clone(),
        }
        .into());
    }

    // Verify proofs
    {
        let client_id_on_b = conn_end_on_b.client_id();
        let client_state_of_a_on_b = ctx_b.client_state(client_id_on_b)?;
        let client_cons_state_path_on_b =
            ClientConsensusStatePath::new(client_id_on_b, &msg.proof_height_on_a);
        let consensus_state_of_a_on_b = ctx_b.consensus_state(&client_cons_state_path_on_b)?;
        let prefix_on_a = conn_end_on_b.counterparty().prefix();
        let port_id_on_a = &chan_end_on_b.counterparty().port_id;
        let chan_id_on_a = chan_end_on_b
            .counterparty()
            .channel_id()
            .ok_or(ChannelError::InvalidCounterpartyChannelId)?;
        let conn_id_on_a = conn_end_on_b.counterparty().connection_id().ok_or(
            ChannelError::UndefinedConnectionCounterparty {
                connection_id: chan_end_on_b.connection_hops()[0].clone(),
            },
        )?;

        // The client must not be frozen.
        if client_state_of_a_on_b.is_frozen() {
            return Err(ChannelError::FrozenClient {
                client_id: client_id_on_b.clone(),
            }
            .into());
        }

        let expected_chan_end_on_a = ChannelEnd::new(
            State::Open,
            *chan_end_on_b.ordering(),
            Counterparty::new(msg.port_id_on_b.clone(), Some(msg.chan_id_on_b.clone())),
            vec![conn_id_on_a.clone()],
            chan_end_on_b.version.clone(),
        );
        let chan_end_path_on_a = ChannelEndPath::new(port_id_on_a, chan_id_on_a);

        // Verify the proof for the channel state against the expected channel end.
        // A counterparty channel id of None in not possible, and is checked in msg.
        client_state_of_a_on_b
            .verify_channel_state(
                msg.proof_height_on_a,
                prefix_on_a,
                &msg.proof_chan_end_on_a,
                consensus_state_of_a_on_b.root(),
                &chan_end_path_on_a,
                &expected_chan_end_on_a,
            )
            .map_err(ChannelError::VerifyChannelFailed)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::core::ics04_channel::handler::chan_open_confirm::validate;
    use crate::core::ics24_host::identifier::ChannelId;
    use crate::prelude::*;
    use rstest::*;
    use test_log::test;

    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::msgs::test_util::get_dummy_raw_counterparty;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order, State};
    use crate::core::ics04_channel::msgs::chan_open_confirm::test_util::get_dummy_raw_msg_chan_open_confirm;
    use crate::core::ics04_channel::msgs::chan_open_confirm::MsgChannelOpenConfirm;
    use crate::core::ics04_channel::Version;
    use crate::core::ics24_host::identifier::{ClientId, ConnectionId};
    use crate::mock::client_state::client_type as mock_client_type;
    use crate::mock::context::MockContext;
    use crate::timestamp::ZERO_DURATION;
    use crate::Height;

    pub struct Fixture {
        pub context: MockContext,
        pub msg: MsgChannelOpenConfirm,
        pub client_id_on_b: ClientId,
        pub conn_id_on_b: ConnectionId,
        pub conn_end_on_b: ConnectionEnd,
        pub chan_end_on_b: ChannelEnd,
        pub proof_height: u64,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let proof_height = 10;
        let context = MockContext::default();

        let client_id_on_b = ClientId::new(mock_client_type(), 45).unwrap();
        let conn_id_on_b = ConnectionId::new(2);
        let conn_end_on_b = ConnectionEnd::new(
            ConnectionState::Open,
            client_id_on_b.clone(),
            ConnectionCounterparty::try_from(get_dummy_raw_counterparty(Some(0))).unwrap(),
            get_compatible_versions(),
            ZERO_DURATION,
        );

        let msg =
            MsgChannelOpenConfirm::try_from(get_dummy_raw_msg_chan_open_confirm(proof_height))
                .unwrap();

        let chan_end_on_b = ChannelEnd::new(
            State::TryOpen,
            Order::Unordered,
            Counterparty::new(msg.port_id_on_b.clone(), Some(ChannelId::default())),
            vec![conn_id_on_b.clone()],
            Version::default(),
        );

        Fixture {
            context,
            msg,
            client_id_on_b,
            conn_id_on_b,
            conn_end_on_b,
            chan_end_on_b,
            proof_height,
        }
    }

    #[rstest]
    fn chan_open_confirm_fail_no_channel(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            client_id_on_b,
            conn_id_on_b,
            conn_end_on_b,
            proof_height,
            ..
        } = fixture;
        let context = context
            .with_client(&client_id_on_b, Height::new(0, proof_height).unwrap())
            .with_connection(conn_id_on_b, conn_end_on_b);

        let res = validate(&context, &msg);

        assert!(
            res.is_err(),
            "Validation fails because no channel exists in the context"
        )
    }

    #[rstest]
    fn chan_open_confirm_fail_channel_wrong_state(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            client_id_on_b,
            conn_id_on_b,
            conn_end_on_b,
            proof_height,
            ..
        } = fixture;

        let wrong_chan_end = ChannelEnd::new(
            State::Init,
            Order::Unordered,
            Counterparty::new(msg.port_id_on_b.clone(), Some(ChannelId::default())),
            vec![conn_id_on_b.clone()],
            Version::default(),
        );
        let context = context
            .with_client(&client_id_on_b, Height::new(0, proof_height).unwrap())
            .with_connection(conn_id_on_b, conn_end_on_b)
            .with_channel(
                msg.port_id_on_b.clone(),
                ChannelId::default(),
                wrong_chan_end,
            );

        let res = validate(&context, &msg);

        assert!(
            res.is_err(),
            "Validation fails because channel is in the wrong state"
        )
    }

    #[rstest]
    fn chan_open_confirm_happy_path(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            client_id_on_b,
            conn_id_on_b,
            conn_end_on_b,
            chan_end_on_b,
            proof_height,
            ..
        } = fixture;

        let context = context
            .with_client(&client_id_on_b, Height::new(0, proof_height).unwrap())
            .with_connection(conn_id_on_b, conn_end_on_b)
            .with_channel(
                msg.port_id_on_b.clone(),
                ChannelId::default(),
                chan_end_on_b,
            );

        let res = validate(&context, &msg);

        assert!(res.is_ok(), "Validation happy path")
    }
}
