use alloc::string::ToString;

use ibc_core::channel::handler::send_packet;
use ibc_core::channel::types::packet::Packet;
use ibc_core::channel::types::timeout::TimeoutHeight;
use ibc_core::handler::types::events::MessageEvent;
use ibc_core::host::types::path::{ChannelEndPath, SeqSendPath};
use ibc_core::primitives::Timestamp;

use crate::context::InterchainAccountExecutionContext;
use crate::controller::msgs::MsgSendTx;
use crate::error::InterchainAccountError;
use crate::port::new_controller_port_id;
use crate::MODULE_ID_STR;

/// Processes a pre-built packet data containing messages to be executed on the
/// host chain
///
/// Note: if the packet is timed out, the channel will be closed. In the case of
/// channel closure, a new channel may be reopened to reconnect to the host chain.
pub fn send_tx<Ctx>(ctx_a: &mut Ctx, msg: MsgSendTx) -> Result<(), InterchainAccountError>
where
    Ctx: InterchainAccountExecutionContext,
{
    send_tx_validate(ctx_a, msg.clone())?;
    send_tx_execute(ctx_a, msg)
}

/// Validate interchain account send tx messages.
fn send_tx_validate<Ctx>(ctx_a: &Ctx, msg: MsgSendTx) -> Result<(), InterchainAccountError>
where
    Ctx: InterchainAccountExecutionContext,
{
    ctx_a.validate_message_signer(&msg.owner)?;

    let port_id = new_controller_port_id(&msg.owner)?;

    let host_timestamp = ctx_a.host_timestamp()?;

    let absolute_timestamp = calc_absolute_timeout(&host_timestamp, &msg.relative_timeout)?;

    // TODO: Why need this?
    // Verifies that the packet is not expired
    // This assumes time synchrony to a certain degree between the controller and counterparty host chain.
    if absolute_timestamp > host_timestamp {
        return Err(InterchainAccountError::invalid("timeout is in the past"));
    }

    ctx_a.get_active_channel_id(&msg.conn_id_on_a, &port_id)?;

    Ok(())
}

/// Execute interchain account send tx messages.
fn send_tx_execute<Ctx>(ctx_a: &mut Ctx, msg: MsgSendTx) -> Result<(), InterchainAccountError>
where
    Ctx: InterchainAccountExecutionContext,
{
    let port_id_on_a = new_controller_port_id(&msg.owner)?;

    let active_channel_id = ctx_a.get_active_channel_id(&msg.conn_id_on_a, &port_id_on_a)?;

    let chan_end_path_on_a = ChannelEndPath::new(&port_id_on_a, &active_channel_id);

    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    let port_id_on_b = chan_end_on_a.counterparty().port_id();

    let chan_id_on_b =
        chan_end_on_a
            .counterparty()
            .channel_id()
            .ok_or(InterchainAccountError::empty(
                "channel id on counterparty is not set",
            ))?;

    let seq_send_path_on_a = SeqSendPath::new(&port_id_on_a, &active_channel_id);

    let seq_on_a = ctx_a.get_next_sequence_send(&seq_send_path_on_a)?;

    let host_timestamp = ctx_a.host_timestamp()?;

    let absolute_timestamp = calc_absolute_timeout(&host_timestamp, &msg.relative_timeout)?;

    let packet = Packet {
        seq_on_a,
        port_id_on_a,
        chan_id_on_a: active_channel_id,
        port_id_on_b: port_id_on_b.clone(),
        chan_id_on_b: chan_id_on_b.clone(),
        data: msg.packet_data.data,
        timeout_height_on_b: TimeoutHeight::Never,
        timeout_timestamp_on_b: absolute_timestamp,
    };

    send_packet(ctx_a, packet)?;

    ctx_a.emit_ibc_event(MessageEvent::Module(MODULE_ID_STR.to_string()).into());

    Ok(())
}

fn calc_absolute_timeout(
    host_timestamp: &Timestamp,
    msg_relative_timestamp: &Timestamp,
) -> Result<Timestamp, InterchainAccountError> {
    let host_timestamp = host_timestamp.nanoseconds();

    let absolute_nanos = host_timestamp
        .checked_add(msg_relative_timestamp.nanoseconds())
        .ok_or(InterchainAccountError::invalid(
            "timeout is too large and overflows",
        ))?;

    let absolute_timestamp =
        Timestamp::from_nanoseconds(absolute_nanos).map_err(InterchainAccountError::source)?;

    Ok(absolute_timestamp)
}
