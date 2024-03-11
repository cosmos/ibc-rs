//! Protocol logic for processing ICS02 messages of type `MsgRecoverClient`.

use ibc_core_client_context::client_state::{
    ClientStateCommon, ClientStateExecution, ClientStateValidation,
};
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

    let subject_client_state = ctx.client_state(&subject_client_id)?;
    let substitute_client_state = ctx.client_state(&substitute_client_id)?;

    let subject_height = subject_client_state.latest_height();
    let substitute_height = substitute_client_state.latest_height();

    if subject_height >= substitute_height {
        return Err(ClientError::ClientRecoveryHeightMismatch {
            subject_height,
            substitute_height,
        }
        .into());
    }

    // also check that the subject and substitute consensus state and consensus state metadata
    // values are all present in the store
    //
    // how does ibc-go's governance proposal work in general?
    // ibc-go does not perform the following two checks; perhaps these checks are being done in the
    // governance module and ibc-go is trusting that these checks were performed
    // need to validate these assumptions
    // check that the commitment root is not empty on the substitute consensus state
    // check that the timestamp of the substitute consensus state > timestamp of the subject consensus state

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
    subject_client_state
        .check_substitute(ctx.get_client_validation_context(), substitute_client_state)?;

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

    let subject_client_state = ctx.client_state(&subject_client_id)?;
    let substitute_client_state = ctx.client_state(&substitute_client_id)?;

    // where and how is a client's `Status` set?
    //    `Status` is set at the ics07 level
    // how to fetch the substitute client's consensus state?
    //    also need consensus metadata, i.e. processed height and processed time
    // is using `ClientState::initialise` a viable way of overwriting the subject client's state?
    // does `ClientState::update_state`'s function signature lend itself to overwriting the client state values that we need to overwrite as part of client recovery?
    // how are race conditions between incrementing client counters avoided?

    let substitute_height = substitute_client_state.latest_height();
    let substitute_consensus_state_path = ClientConsensusStatePath::new(
        substitute_client_id,
        substitute_height.revision_number(),
        substitute_height.revision_height(),
    );
    let substitute_consensus_state = ctx.consensus_state(&substitute_consensus_state_path)?;

    subject_client_state.initialise(
        ctx.get_client_execution_context(),
        &substitute_client_id,
        substitute_consensus_state,
    )?;

    subject_client_state.update_on_recovery(ctx.get_client_execution_context())?;

    Ok(())
}
