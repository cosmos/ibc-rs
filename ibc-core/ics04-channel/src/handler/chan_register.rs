//! Protocol logic specific to ICS4 messages of type `MsgChannelOpenTry`.

use core::time::Duration;

use ibc_core_channel_types::channel::{
    ChannelEnd, Counterparty as ChannelCounterparty, State as ChannelState,
};
use ibc_core_channel_types::error::ChannelError;
use ibc_core_channel_types::msgs::MsgChannelRegister;
use ibc_core_connection::types::{
    ConnectionEnd, Counterparty as ConnectionCounterparty, State as ConnectionState,
};
use ibc_core_host::types::identifiers::{ChannelId, ConnectionId};
use ibc_core_host::types::path::{
    ChannelEndPath, ClientConnectionPath, ConnectionPath, SeqAckPath, SeqRecvPath, SeqSendPath,
};
use ibc_core_host::{ExecutionContext, ValidationContext};
use ibc_core_router::module::Module;
use ibc_primitives::prelude::*;

pub fn chan_register_validate<ValCtx>(
    ctx: &ValCtx,
    _module: &dyn Module,
    msg: MsgChannelRegister,
) -> Result<(), ChannelError>
where
    ValCtx: ValidationContext,
{
    ctx.validate_message_signer(&msg.signer)?;

    // TODO(rano): perform validation
    // or we just skip validation as it is socially validated via governance

    // validate(ctx, &msg)?;

    Ok(())
}

pub fn chan_register_execute<ExecCtx>(
    ctx: &mut ExecCtx,
    _module: &mut dyn Module,
    msg: MsgChannelRegister,
) -> Result<(), ChannelError>
where
    ExecCtx: ExecutionContext,
{
    // TODO(rano): store counters for send/recv/ack packets

    // bootstrap connection end

    let versions = ctx.get_compatible_versions();

    let conn_end_on_a = ConnectionEnd::new(
        ConnectionState::Open,
        msg.client_id_on_a.clone(),
        ConnectionCounterparty::new(
            msg.client_id_on_b.clone(),
            Some(ConnectionId::V2(msg.client_id_on_b.clone())),
            msg.commitment_prefix_on_b.clone(),
        ),
        versions,
        Duration::from_secs(0),
    )?;

    // Construct the identifier for the new connection.
    let conn_id_on_a = ConnectionId::V2(msg.client_id_on_a.clone());

    ctx.store_connection_to_client(
        &ClientConnectionPath::new(msg.client_id_on_a.clone()),
        conn_id_on_a.clone(),
    )?;
    ctx.store_connection(&ConnectionPath::new(&conn_id_on_a), conn_end_on_a)?;

    // bootstrap channel end

    let chan_end_on_a = ChannelEnd::new(
        ChannelState::Open,
        msg.ordering,
        ChannelCounterparty::new(
            msg.port_id_on_b.clone(),
            Some(ChannelId::V2(msg.client_id_on_b.clone())),
        ),
        vec![conn_id_on_a],
        msg.version_proposal.clone(),
    )?;

    let chan_end_path_on_a = ChannelEndPath::new(
        &msg.port_id_on_a,
        &ChannelId::V2(msg.client_id_on_a.clone()),
    );
    ctx.store_channel(&chan_end_path_on_a, chan_end_on_a)?;

    // Initialize send, recv, and ack sequence numbers.
    let seq_send_path = SeqSendPath::new(
        &msg.port_id_on_a,
        &ChannelId::V2(msg.client_id_on_a.clone()),
    );
    ctx.store_next_sequence_send(&seq_send_path, 1.into())?;

    let seq_recv_path = SeqRecvPath::new(
        &msg.port_id_on_a,
        &ChannelId::V2(msg.client_id_on_a.clone()),
    );
    ctx.store_next_sequence_recv(&seq_recv_path, 1.into())?;

    let seq_ack_path = SeqAckPath::new(
        &msg.port_id_on_a,
        &ChannelId::V2(msg.client_id_on_a.clone()),
    );
    ctx.store_next_sequence_ack(&seq_ack_path, 1.into())?;

    Ok(())
}

// fn validate<Ctx>(ctx: &Ctx, msg: &MsgChannelRegister) -> Result<(), ChannelError>
// where
//     Ctx: ValidationContext,
// {
//     ctx.validate_message_signer(&msg.signer)?;

//     msg.verify_connection_hops_length()?;

//     let conn_end_on_b = ctx.connection_end(&msg.connection_hops_on_b[0])?;

//     conn_end_on_b.verify_state_matches(&ConnectionState::Open)?;

//     let conn_version = conn_end_on_b.versions();

//     conn_version[0].verify_feature_supported(msg.ordering.to_string())?;

//     // Verify proofs
//     {
//         let client_id_on_b = conn_end_on_b.client_id();
//         let client_val_ctx_b = ctx.get_client_validation_context();
//         let client_state_of_a_on_b = client_val_ctx_b.client_state(client_id_on_b)?;

//         client_state_of_a_on_b
//             .status(ctx.get_client_validation_context(), client_id_on_b)?
//             .verify_is_active()?;

//         client_state_of_a_on_b.validate_proof_height(msg.proof_height_on_a)?;

//         let client_cons_state_path_on_b = ClientConsensusStatePath::new(
//             client_id_on_b.clone(),
//             msg.proof_height_on_a.revision_number(),
//             msg.proof_height_on_a.revision_height(),
//         );
//         let consensus_state_of_a_on_b =
//             client_val_ctx_b.consensus_state(&client_cons_state_path_on_b)?;
//         let prefix_on_a = conn_end_on_b.counterparty().prefix();
//         let port_id_on_a = msg.port_id_on_a.clone();
//         let chan_id_on_a = msg.chan_id_on_a.clone();
//         let conn_id_on_a = conn_end_on_b
//             .counterparty()
//             .connection_id()
//             .ok_or(ConnectionError::MissingCounterparty)?;

//         let expected_chan_end_on_a = ChannelEnd::new(
//             ChannelState::Init,
//             msg.ordering,
//             Counterparty::new(msg.port_id_on_b.clone(), None),
//             vec![conn_id_on_a.clone()],
//             msg.version_supported_on_a.clone(),
//         )?;
//         let chan_end_path_on_a = ChannelEndPath::new(&port_id_on_a, &chan_id_on_a);

//         // Verify the proof for the channel state against the expected channel end.
//         // A counterparty channel id of None in not possible, and is checked by validate_basic in msg.
//         client_state_of_a_on_b.verify_membership(
//             prefix_on_a,
//             &msg.proof_chan_end_on_a,
//             consensus_state_of_a_on_b.root(),
//             Path::ChannelEnd(chan_end_path_on_a),
//             expected_chan_end_on_a.encode_vec(),
//         )?;
//     }

//     Ok(())
// }
