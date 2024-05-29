use ibc_core_channel::handler::{
    acknowledgement_packet_execute, acknowledgement_packet_validate, chan_close_confirm_execute,
    chan_close_confirm_validate, chan_close_init_execute, chan_close_init_validate,
    chan_open_ack_execute, chan_open_ack_validate, chan_open_confirm_execute,
    chan_open_confirm_validate, chan_open_init_execute, chan_open_init_validate,
    chan_open_try_execute, chan_open_try_validate, recv_packet_execute, recv_packet_validate,
    timeout_packet_execute, timeout_packet_validate, TimeoutMsgType,
};
use ibc_core_channel::types::msgs::{
    channel_msg_to_port_id, packet_msg_to_port_id, ChannelMsg, PacketMsg,
};
use ibc_core_client::context::{ClientExecutionContext, ClientValidationContext};
use ibc_core_client::handler::{create_client, update_client, upgrade_client};
use ibc_core_client::types::error::ClientError;
use ibc_core_client::types::msgs::{ClientMsg, MsgUpdateOrMisbehaviour};
use ibc_core_connection::handler::{
    conn_open_ack, conn_open_confirm, conn_open_init, conn_open_try,
};
use ibc_core_connection::types::msgs::ConnectionMsg;
use ibc_core_handler_types::error::ContextError;
use ibc_core_handler_types::msgs::MsgEnvelope;
use ibc_core_host::{ExecutionContext, ValidationContext};
use ibc_core_router::router::Router;
use ibc_core_router::types::error::RouterError;
use ibc_primitives::proto::Any;

/// Entrypoint which performs both validation and message execution
pub fn dispatch<Ctx>(
    ctx: &mut Ctx,
    router: &mut impl Router,
    msg: MsgEnvelope,
) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
    <<Ctx::V as ClientValidationContext>::ClientStateRef as TryFrom<Any>>::Error: Into<ClientError>,
    <<Ctx::E as ClientExecutionContext>::ClientStateMut as TryFrom<Any>>::Error: Into<ClientError>,
    <Ctx::HostClientState as TryFrom<Any>>::Error: Into<ClientError>,
{
    validate(ctx, router, msg.clone())?;
    execute(ctx, router, msg)
}

/// Entrypoint which only performs message validation
///
/// If a transaction contains `n` messages `m_1` ... `m_n`, then
/// they MUST be processed as follows:
///     validate(m_1), execute(m_1), ..., validate(m_n), execute(m_n)
/// That is, the state transition of message `i` must be applied before
/// message `i+1` is validated. This is equivalent to calling
/// `dispatch()` on each successively.
pub fn validate<Ctx>(ctx: &Ctx, router: &impl Router, msg: MsgEnvelope) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
    <<Ctx::V as ClientValidationContext>::ClientStateRef as TryFrom<Any>>::Error: Into<ClientError>,
    <Ctx::HostClientState as TryFrom<Any>>::Error: Into<ClientError>,
{
    match msg {
        MsgEnvelope::Client(msg) => match msg {
            ClientMsg::CreateClient(msg) => create_client::validate(ctx, msg),
            ClientMsg::UpdateClient(msg) => {
                update_client::validate(ctx, MsgUpdateOrMisbehaviour::UpdateClient(msg))
            }
            ClientMsg::Misbehaviour(msg) => {
                update_client::validate(ctx, MsgUpdateOrMisbehaviour::Misbehaviour(msg))
            }
            ClientMsg::UpgradeClient(msg) => upgrade_client::validate(ctx, msg),
            ClientMsg::RecoverClient(_msg) => {
                // Recover client messages are not dispatched by ibc-rs as they can only be
                // authorized via a passing governance proposal
                Ok(())
            }
        },
        MsgEnvelope::Connection(msg) => match msg {
            ConnectionMsg::OpenInit(msg) => conn_open_init::validate(ctx, msg),
            ConnectionMsg::OpenTry(msg) => conn_open_try::validate(ctx, msg),
            ConnectionMsg::OpenAck(msg) => conn_open_ack::validate(ctx, msg),
            ConnectionMsg::OpenConfirm(msg) => conn_open_confirm::validate(ctx, &msg),
        },
        MsgEnvelope::Channel(msg) => {
            let port_id = channel_msg_to_port_id(&msg);
            let module_id = router
                .lookup_module(port_id)
                .ok_or(RouterError::UnknownPort {
                    port_id: port_id.clone(),
                })?;
            let module = router
                .get_route(&module_id)
                .ok_or(RouterError::ModuleNotFound)?;

            match msg {
                ChannelMsg::OpenInit(msg) => chan_open_init_validate(ctx, module, msg),
                ChannelMsg::OpenTry(msg) => chan_open_try_validate(ctx, module, msg),
                ChannelMsg::OpenAck(msg) => chan_open_ack_validate(ctx, module, msg),
                ChannelMsg::OpenConfirm(msg) => chan_open_confirm_validate(ctx, module, msg),
                ChannelMsg::CloseInit(msg) => chan_close_init_validate(ctx, module, msg),
                ChannelMsg::CloseConfirm(msg) => chan_close_confirm_validate(ctx, module, msg),
            }
        }
        MsgEnvelope::Packet(msg) => {
            let port_id = packet_msg_to_port_id(&msg);
            let module_id = router
                .lookup_module(port_id)
                .ok_or(RouterError::UnknownPort {
                    port_id: port_id.clone(),
                })?;
            let module = router
                .get_route(&module_id)
                .ok_or(RouterError::ModuleNotFound)?;

            match msg {
                PacketMsg::Recv(msg) => recv_packet_validate(ctx, msg),
                PacketMsg::Ack(msg) => acknowledgement_packet_validate(ctx, module, msg),
                PacketMsg::Timeout(msg) => {
                    timeout_packet_validate(ctx, module, TimeoutMsgType::Timeout(msg))
                }
                PacketMsg::TimeoutOnClose(msg) => {
                    timeout_packet_validate(ctx, module, TimeoutMsgType::TimeoutOnClose(msg))
                }
            }
        }
    }
}

/// Entrypoint which only performs message execution
pub fn execute<Ctx>(
    ctx: &mut Ctx,
    router: &mut impl Router,
    msg: MsgEnvelope,
) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
    <<Ctx::E as ClientExecutionContext>::ClientStateMut as TryFrom<Any>>::Error: Into<ClientError>,
{
    match msg {
        MsgEnvelope::Client(msg) => match msg {
            ClientMsg::CreateClient(msg) => create_client::execute(ctx, msg),
            ClientMsg::UpdateClient(msg) => {
                update_client::execute(ctx, MsgUpdateOrMisbehaviour::UpdateClient(msg))
            }
            ClientMsg::Misbehaviour(msg) => {
                update_client::execute(ctx, MsgUpdateOrMisbehaviour::Misbehaviour(msg))
            }
            ClientMsg::UpgradeClient(msg) => upgrade_client::execute(ctx, msg),
            ClientMsg::RecoverClient(_msg) => {
                // Recover client messages are not dispatched by ibc-rs as they can only be
                // authorized via a passing governance proposal
                Ok(())
            }
        },
        MsgEnvelope::Connection(msg) => match msg {
            ConnectionMsg::OpenInit(msg) => conn_open_init::execute(ctx, msg),
            ConnectionMsg::OpenTry(msg) => conn_open_try::execute(ctx, msg),
            ConnectionMsg::OpenAck(msg) => conn_open_ack::execute(ctx, msg),
            ConnectionMsg::OpenConfirm(msg) => conn_open_confirm::execute(ctx, &msg),
        },
        MsgEnvelope::Channel(msg) => {
            let port_id = channel_msg_to_port_id(&msg);
            let module_id = router
                .lookup_module(port_id)
                .ok_or(RouterError::UnknownPort {
                    port_id: port_id.clone(),
                })?;
            let module = router
                .get_route_mut(&module_id)
                .ok_or(RouterError::ModuleNotFound)?;

            match msg {
                ChannelMsg::OpenInit(msg) => chan_open_init_execute(ctx, module, msg),
                ChannelMsg::OpenTry(msg) => chan_open_try_execute(ctx, module, msg),
                ChannelMsg::OpenAck(msg) => chan_open_ack_execute(ctx, module, msg),
                ChannelMsg::OpenConfirm(msg) => chan_open_confirm_execute(ctx, module, msg),
                ChannelMsg::CloseInit(msg) => chan_close_init_execute(ctx, module, msg),
                ChannelMsg::CloseConfirm(msg) => chan_close_confirm_execute(ctx, module, msg),
            }
        }
        MsgEnvelope::Packet(msg) => {
            let port_id = packet_msg_to_port_id(&msg);
            let module_id = router
                .lookup_module(port_id)
                .ok_or(RouterError::UnknownPort {
                    port_id: port_id.clone(),
                })?;
            let module = router
                .get_route_mut(&module_id)
                .ok_or(RouterError::ModuleNotFound)?;

            match msg {
                PacketMsg::Recv(msg) => recv_packet_execute(ctx, module, msg),
                PacketMsg::Ack(msg) => acknowledgement_packet_execute(ctx, module, msg),
                PacketMsg::Timeout(msg) => {
                    timeout_packet_execute(ctx, module, TimeoutMsgType::Timeout(msg))
                }
                PacketMsg::TimeoutOnClose(msg) => {
                    timeout_packet_execute(ctx, module, TimeoutMsgType::TimeoutOnClose(msg))
                }
            }
        }
    }
}
