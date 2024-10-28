//! Protocol logic specific to processing ICS2 messages of type `MsgProvideCouterparty`.

use ibc_eureka_core_client_context::prelude::*;
use ibc_eureka_core_client_types::error::ClientError;
use ibc_eureka_core_client_types::msgs::MsgProvideCouterparty;
use ibc_eureka_core_host::{ExecutionContext, ValidationContext};
use ibc_primitives::prelude::*;

pub fn validate<Ctx>(ctx: &Ctx, msg: MsgProvideCouterparty) -> Result<(), ClientError>
where
    Ctx: ValidationContext,
{
    let MsgProvideCouterparty {
        client_id, signer, ..
    } = &msg;

    ctx.validate_message_signer(signer)?;

    let client_val_ctx = ctx.get_client_validation_context();

    // Read client state from the host chain store. The client should already exist.
    let client_state = client_val_ctx.client_state(client_id)?;

    client_state
        .status(client_val_ctx, client_id)?
        .verify_is_active()?;

    if client_val_ctx.counterparty_meta(client_id)?.is_some() {
        return Err(ClientError::ClientSpecific {
            description: "counterparty is already provided".into(),
        });
    }

    Ok(())
}

pub fn execute<Ctx>(ctx: &mut Ctx, msg: MsgProvideCouterparty) -> Result<(), ClientError>
where
    Ctx: ExecutionContext,
{
    let MsgProvideCouterparty {
        client_id,
        counterparty_client_id,
        counterparty_commitment_prefix,
        ..
    } = &msg;

    let client_exec_ctx = ctx.get_client_execution_context();

    client_exec_ctx.store_counterparty_meta(
        client_id,
        counterparty_client_id,
        counterparty_commitment_prefix,
    )?;

    Ok(())
}
