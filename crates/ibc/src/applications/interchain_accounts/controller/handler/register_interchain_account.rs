use crate::prelude::*;

use crate::applications::interchain_accounts::context::InterchainAccountExecutionContext;
use crate::applications::interchain_accounts::context::InterchainAccountValidationContext;
use crate::applications::interchain_accounts::controller::msgs::MsgRegisterInterchainAccount;
use crate::applications::interchain_accounts::error::InterchainAccountError;
use crate::applications::interchain_accounts::port::default_host_port_id;
use crate::applications::interchain_accounts::port::new_controller_port_id;
use crate::core::ics04_channel::channel::Order;
use crate::core::ics04_channel::handler::chan_open_init::chan_open_init_execute;
use crate::core::ics04_channel::handler::chan_open_init::chan_open_init_validate;
use crate::core::ics04_channel::msgs::MsgChannelOpenInit;
use crate::core::ics24_host::identifier::ChannelId;
use crate::core::ics24_host::path::ChannelEndPath;

/// Entry point for registering an interchain account.
///
/// - Generates a new `PortId` using the provided owner and returns an error if
///   the port is already in use.
/// - Callers are expected to provide the appropriate application version. For
///   example, this could be an ICS27 encoded metadata type with a nested
///   application version.
/// - Gaining access to interchain accounts whose channels have closed cannot be
///   done with this function. A regular MsgChannelOpenInit must be used.
pub fn register_interchain_account<Ctx>(
    ctx_a: &mut Ctx,
    msg: MsgRegisterInterchainAccount,
) -> Result<(), InterchainAccountError>
where
    Ctx: InterchainAccountExecutionContext,
{
    register_interchain_account_validate(ctx_a, &msg)?;
    register_interchain_account_execute(ctx_a, msg)
}

/// Validate interchain account registration message.
pub fn register_interchain_account_validate<ValCtx>(
    ctx_a: &ValCtx,
    msg: &MsgRegisterInterchainAccount,
) -> Result<(), InterchainAccountError>
where
    ValCtx: InterchainAccountValidationContext,
{
    ctx_a.validate_message_signer(&msg.owner)?;

    let port_id_on_a = new_controller_port_id(&msg.owner)?;

    if let Ok(active_chan_id_on_a) = ctx_a.get_active_channel_id(&msg.conn_id_on_a, &port_id_on_a) {
        let chan_end_path_on_a = ChannelEndPath::new(&port_id_on_a, &active_chan_id_on_a);

        let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

        if !chan_end_on_a.is_closed() {
            return Err(InterchainAccountError::already_exists(
                "channel is already active or a handshake is in flight",
            ));
        }
    }

    let module_id = ctx_a
        .lookup_module_by_port(&port_id_on_a)
        .ok_or(InterchainAccountError::not_found("no module found").given(&port_id_on_a))?;

    let port_id_on_b = default_host_port_id()?;

    let msg_chan_open_init = MsgChannelOpenInit::new(
        port_id_on_a,
        vec![msg.conn_id_on_a.clone()],
        port_id_on_b,
        Order::Ordered,
        msg.owner.clone(),
        msg.version.clone(),
    );

    if let Err(e) = chan_open_init_validate(ctx_a, module_id, msg_chan_open_init) {
        // TODO: uncomment the below line after refactoring the logger to get
        // accessible from the context.
        //
        // ctx_a.log_message(format!( "error registering interchain account.
        //     Error: {}", e ));
        return Err(InterchainAccountError::source(e));
    }

    Ok(())
}

/// Execute interchain account registration message.
pub fn register_interchain_account_execute<ExecCtx>(
    ctx_a: &mut ExecCtx,
    msg: MsgRegisterInterchainAccount,
) -> Result<(), InterchainAccountError>
where
    ExecCtx: InterchainAccountExecutionContext,
{
    let port_id_on_a = new_controller_port_id(&msg.owner)?;

    let module_id = ctx_a
        .lookup_module_by_port(&port_id_on_a)
        .ok_or(InterchainAccountError::not_found("no module found").given(&port_id_on_a))?;

    let port_id_on_b = default_host_port_id()?;

    let msg_chan_open_init = MsgChannelOpenInit::new(
        port_id_on_a,
        vec![msg.conn_id_on_a.clone()],
        port_id_on_b,
        Order::Ordered,
        msg.owner.clone(),
        msg.version,
    );

    if let Err(e) = chan_open_init_execute(ctx_a, module_id, msg_chan_open_init) {
        ctx_a.log_message(format!(
            "error registering interchain account. Error: {}",
            e
        ));
        return Err(InterchainAccountError::source(e));
    }

    let chan_counter_on_a = ctx_a.channel_counter()?;

    let chan_id_on_a = ChannelId::new(chan_counter_on_a);

    ctx_a.log_message(format!(
        "successfully registered interchain account with channel id: {}",
        chan_id_on_a
    ));

    Ok(())
}
