//! Protocol logic for processing ICS02 messages of type `MsgRecoverClient`.

use ibc_core_client_context::client_state::{
    ClientStateCommon, ClientStateExecution, ClientStateValidation,
};
use ibc_core_client_types::msgs::MsgRecoverClient;
use ibc_core_handler_types::error::ContextError;
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

    let subject_client_state = ctx.client_state(&subject_client_id)?;
    let substitute_client_state = ctx.client_state(&substitute_client_id)?;

    substitute_client_state
        .status(ctx.get_client_validation_context(), &substitute_client_id)?
        .verify_is_active()?;

    // Verify that the subject client is inactive, i.e., that it is either frozen or expired
    subject_client_state
        .status(ctx.get_client_validation_context(), &subject_client_id)?
        .verify_is_inactive()?;

    Ok(())
}

pub fn execute<Ctx>(ctx: &mut Ctx, msg: MsgRecoverClient) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    Ok(())
}
