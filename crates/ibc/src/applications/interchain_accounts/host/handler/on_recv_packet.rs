use alloc::vec::Vec;
use ibc_proto::protobuf::Protobuf;

use crate::applications::interchain_accounts::context::InterchainAccountExecutionContext;
use crate::applications::interchain_accounts::error::InterchainAccountError;
use crate::applications::interchain_accounts::host::msgs::cosmos_tx::CosmosTx;
use crate::applications::interchain_accounts::packet::InterchainAccountPacketData;

use crate::core::execute;
use crate::core::ics04_channel::packet::Packet;
use crate::core::ics24_host::identifier::ConnectionId;
use crate::core::ics24_host::identifier::PortId;
use crate::core::ics24_host::path::ChannelEndPath;
use crate::core::validate;
use crate::core::MsgEnvelope;

/// Handles a given interchain accounts packet on a destination host chain.
/// If the transaction is successfully executed, the transaction response bytes will be returned.
pub fn on_recv_packet<Ctx>(ctx_b: &mut Ctx, packet: &Packet) -> Result<(), InterchainAccountError>
where
    Ctx: InterchainAccountExecutionContext,
{
    let ica_packet_date = InterchainAccountPacketData::try_from(packet.data.clone())?;

    let cosmos_tx =
        CosmosTx::decode_vec(&ica_packet_date.data).map_err(InterchainAccountError::source)?;

    let chan_end_path_on_b = ChannelEndPath::new(&packet.port_id_on_b, &packet.chan_id_on_b);

    let chan_end_on_b = ctx_b.channel_end(&chan_end_path_on_b)?;

    let mut envelope_msgs: Vec<MsgEnvelope> = Vec::new();

    for msg in cosmos_tx.messages {
        let envelope = msg
            .try_into()
            .map_err(|_| InterchainAccountError::invalid("msg is not of the MsgEnvelope type"))?;
        envelope_msgs.push(envelope);
    }

    validate_msgs(
        ctx_b,
        &envelope_msgs,
        &chan_end_on_b.connection_hops,
        &packet.port_id_on_a,
    )?;

    execute_msgs(ctx_b, envelope_msgs)?;

    Ok(())
}

/// Validates the provided msgs contain the correct interchain account signer
/// address retrieved from state using the provided controller port identifier
pub fn validate_msgs<Ctx>(
    ctx_b: &Ctx,
    msgs: &Vec<MsgEnvelope>,
    conn_hops_on_b: &[ConnectionId],
    port_id_on_a: &PortId,
) -> Result<(), InterchainAccountError>
where
    Ctx: InterchainAccountExecutionContext,
{
    ctx_b.get_ica_address(&conn_hops_on_b[0], port_id_on_a)?;

    let params_on_b = ctx_b.get_params()?;

    for msg in msgs {
        if !params_on_b.contains_msg_type(msg) {
            Err(InterchainAccountError::not_allowed(
                "msg type is not allowed",
            ))?;
        }

        ctx_b.validate_message_signer(&msg.signer())?;
    }

    Ok(())
}

/// Handles a given interchain accounts packet on a destination host chain.
/// If the transaction is successfully executed, the transaction response bytes is returned.
pub fn execute_msgs<Ctx>(
    ctx_b: &mut Ctx,
    msgs: Vec<MsgEnvelope>,
) -> Result<(), InterchainAccountError>
where
    Ctx: InterchainAccountExecutionContext,
{
    for msg in msgs {
        validate(ctx_b, msg.clone()).map_err(InterchainAccountError::source)?;

        execute(ctx_b, msg).map_err(InterchainAccountError::source)?;
    }

    Ok(())
}
