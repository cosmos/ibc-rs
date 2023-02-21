//! Protocol logic specific to ICS4 messages of type `MsgChannelOpenTry`.

use crate::core::ics03_connection::connection::State as ConnectionState;
use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, State};
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::msgs::chan_open_try::MsgChannelOpenTry;
use crate::core::ics24_host::path::{ChannelEndPath, ClientConsensusStatePath};
use crate::prelude::*;

use crate::core::{ContextError, ValidationContext};

pub fn validate<Ctx>(ctx_b: &Ctx, msg: &MsgChannelOpenTry) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    // An IBC connection running on the local (host) chain should exist.
    if msg.connection_hops_on_b.len() != 1 {
        return Err(ChannelError::InvalidConnectionHopsLength {
            expected: 1,
            actual: msg.connection_hops_on_b.len(),
        })
        .map_err(ContextError::ChannelError);
    }

    let conn_end_on_b = ctx_b.connection_end(&msg.connection_hops_on_b[0])?;
    if !conn_end_on_b.state_matches(&ConnectionState::Open) {
        return Err(ChannelError::ConnectionNotOpen {
            connection_id: msg.connection_hops_on_b[0].clone(),
        })
        .map_err(ContextError::ChannelError);
    }

    let conn_version = match conn_end_on_b.versions() {
        [version] => version,
        _ => {
            return Err(ChannelError::InvalidVersionLengthConnection)
                .map_err(ContextError::ChannelError)
        }
    };

    let channel_feature = msg.ordering.to_string();
    if !conn_version.is_supported_feature(channel_feature) {
        return Err(ChannelError::ChannelFeatureNotSuportedByConnection)
            .map_err(ContextError::ChannelError);
    }

    // Verify proofs
    {
        let client_id_on_b = conn_end_on_b.client_id();
        let client_state_of_a_on_b = ctx_b.client_state(client_id_on_b)?;
        let client_cons_state_path_on_b =
            ClientConsensusStatePath::new(client_id_on_b, &msg.proof_height_on_a);
        let consensus_state_of_a_on_b = ctx_b.consensus_state(&client_cons_state_path_on_b)?;
        let prefix_on_a = conn_end_on_b.counterparty().prefix();
        let port_id_on_a = msg.port_id_on_a.clone();
        let chan_id_on_a = msg.chan_id_on_a.clone();
        let conn_id_on_a = conn_end_on_b.counterparty().connection_id().ok_or(
            ChannelError::UndefinedConnectionCounterparty {
                connection_id: msg.connection_hops_on_b[0].clone(),
            },
        )?;

        // The client must not be frozen.
        if client_state_of_a_on_b.is_frozen() {
            return Err(ChannelError::FrozenClient {
                client_id: client_id_on_b.clone(),
            })
            .map_err(ContextError::ChannelError);
        }

        let expected_chan_end_on_a = ChannelEnd::new(
            State::Init,
            msg.ordering,
            Counterparty::new(msg.port_id_on_b.clone(), None),
            vec![conn_id_on_a.clone()],
            msg.version_supported_on_a.clone(),
        );
        let chan_end_path_on_a = ChannelEndPath::new(&port_id_on_a, &chan_id_on_a);

        // Verify the proof for the channel state against the expected channel end.
        // A counterparty channel id of None in not possible, and is checked by validate_basic in msg.
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
    use crate::prelude::*;
    use rstest::*;
    use test_log::test;

    use crate::core::ics04_channel::handler::chan_open_try;

    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::msgs::test_util::get_dummy_raw_counterparty;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::msgs::chan_open_try::test_util::get_dummy_raw_msg_chan_open_try;
    use crate::core::ics04_channel::msgs::chan_open_try::MsgChannelOpenTry;
    use crate::core::ics24_host::identifier::{ClientId, ConnectionId};
    use crate::mock::client_state::client_type as mock_client_type;
    use crate::mock::context::MockContext;
    use crate::timestamp::ZERO_DURATION;
    use crate::Height;

    pub struct Fixture {
        pub context: MockContext,
        pub msg: MsgChannelOpenTry,
        pub client_id_on_b: ClientId,
        pub conn_id_on_b: ConnectionId,
        pub conn_end_on_b: ConnectionEnd,
        pub proof_height: u64,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let proof_height = 10;
        let conn_id_on_b = ConnectionId::new(2);
        let client_id_on_b = ClientId::new(mock_client_type(), 45).unwrap();

        // This is the connection underlying the channel we're trying to open.
        let conn_end_on_b = ConnectionEnd::new(
            ConnectionState::Open,
            client_id_on_b.clone(),
            ConnectionCounterparty::try_from(get_dummy_raw_counterparty(Some(0))).unwrap(),
            get_compatible_versions(),
            ZERO_DURATION,
        );

        // We're going to test message processing against this message.
        // Note: we make the counterparty's channel_id `None`.
        let mut msg =
            MsgChannelOpenTry::try_from(get_dummy_raw_msg_chan_open_try(proof_height)).unwrap();

        let hops = vec![conn_id_on_b.clone()];
        msg.connection_hops_on_b = hops;

        let context = MockContext::default();

        Fixture {
            context,
            msg,
            client_id_on_b,
            conn_id_on_b,
            conn_end_on_b,
            proof_height,
        }
    }

    #[rstest]
    fn chan_open_try_fail_no_connection(fixture: Fixture) {
        let Fixture { context, msg, .. } = fixture;

        let res = chan_open_try::validate(&context, &msg);

        assert!(
            res.is_err(),
            "Validation fails because no connection exists in the context"
        )
    }

    #[rstest]
    fn chan_open_try_fail_no_client_state(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            conn_id_on_b,
            conn_end_on_b,
            ..
        } = fixture;
        let context = context.with_connection(conn_id_on_b, conn_end_on_b);

        let res = chan_open_try::validate(&context, &msg);

        assert!(
            res.is_err(),
            "Validation fails because the context has no client state"
        )
    }

    #[rstest]
    fn chan_open_try_happy_path(fixture: Fixture) {
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

        let res = chan_open_try::validate(&context, &msg);

        assert!(res.is_ok(), "Validation success: happy path")
    }
}
