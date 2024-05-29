//! Protocol logic for processing ICS02 messages of type `MsgRecoverClient`.

use ibc_core_client_context::prelude::*;
use ibc_core_client_types::error::ClientError;
use ibc_core_client_types::msgs::MsgRecoverClient;
use ibc_core_handler_types::error::ContextError;
use ibc_core_host::types::path::ClientConsensusStatePath;
use ibc_core_host::{ExecutionContext, ValidationContext};

/// Performs the validation steps associated with the client recovery process. This
/// includes validating that the parameters of the subject and substitute clients match,
/// as well as validating that the substitute client *is* active and that the subject
/// client is *not* active.
pub fn validate<Ctx>(ctx: &Ctx, msg: MsgRecoverClient) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let signer = msg.signer;
    let subject_client_id = msg.subject_client_id.clone();
    let substitute_client_id = msg.substitute_client_id.clone();

    ctx.validate_message_signer(&signer)?;

    let client_val_ctx = ctx.get_client_validation_context();

    let subject_client_state = client_val_ctx.client_state(&subject_client_id)?;
    let substitute_client_state = client_val_ctx.client_state(&substitute_client_id)?;

    let subject_height = subject_client_state.latest_height();
    let substitute_height = substitute_client_state.latest_height();

    if subject_height >= substitute_height {
        return Err(ClientError::ClientRecoveryHeightMismatch {
            subject_height,
            substitute_height,
        }
        .into());
    }

    substitute_client_state
        .status(ctx.get_client_validation_context(), &substitute_client_id)?
        .verify_is_active()?;

    // Verify that the subject client is inactive, i.e., that it is either frozen or expired
    subject_client_state
        .status(ctx.get_client_validation_context(), &subject_client_id)?
        .verify_is_inactive()?;

    // Check that the subject client state and substitute client states match, i.e., that
    // all their respective client state parameters match except for frozen height, latest
    // height, trusting period, and chain ID
    subject_client_state.check_substitute(
        ctx.get_client_validation_context(),
        substitute_client_state.into(),
    )?;

    Ok(())
}

/// Executes the steps needed to recover the subject client, namely:
///  - setting the subject's status from either `frozen` or `expired` to `active`
///  - copying the substitute client's consensus state as the subject's consensus state
///  - setting the subject client's processed height and processed time values to match the substitute client's
///  - setting the subject client's latest height, trusting period, and chain ID values to match the substitute client's
pub fn execute<Ctx>(ctx: &mut Ctx, msg: MsgRecoverClient) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    let subject_client_id = msg.subject_client_id.clone();
    let substitute_client_id = msg.substitute_client_id.clone();

    let client_exec_ctx = ctx.get_client_execution_context();

    let subject_client_state = client_exec_ctx.client_state(&subject_client_id)?;
    let substitute_client_state = client_exec_ctx.client_state(&substitute_client_id)?;
    let substitute_consensus_state =
        client_exec_ctx.consensus_state(&ClientConsensusStatePath::new(
            substitute_client_id.clone(),
            substitute_client_state.latest_height().revision_number(),
            substitute_client_state.latest_height().revision_height(),
        ))?;

    subject_client_state.update_on_recovery(
        ctx.get_client_execution_context(),
        &subject_client_id,
        substitute_client_state.into(),
        substitute_consensus_state.into(),
    )?;

    Ok(())
}
