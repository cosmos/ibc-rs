#[cfg(feature = "val-exec-entry")]
use ibc_proto::google::protobuf::Any;

#[cfg(feature = "val-exec-entry")]
use super::ValidationContext;

use super::{ExecutionContext, MsgEnvelope, RouterError};

use crate::core::handler::{execution_handler, validation_handler};

/// Entrypoint which performs both validation and message execution
pub fn dispatch(ctx: &mut impl ExecutionContext, msg: MsgEnvelope) -> Result<(), RouterError> {
    validation_handler(ctx, msg.clone())?;
    execution_handler(ctx, msg)
}

/// Entrypoint which only performs message validation
///
/// If a transaction contains `n` messages `m_1` ... `m_n`, then
/// they MUST be processed as follows:
///     validate(m_1), execute(m_1), ..., validate(m_n), execute(m_n)
/// That is, the state transition of message `i` must be applied before
/// message `i+1` is validated. This is equivalent to calling
/// `dispatch()` on each successively.
#[cfg(feature = "val-exec-entry")]
pub fn validate<Ctx>(ctx: &Ctx, message: Any) -> Result<(), RouterError>
where
    Ctx: ValidationContext,
{
    let envelope: MsgEnvelope = message.try_into()?;
    validation_handler(ctx, envelope)
}

/// Entrypoint which only performs message execution
#[cfg(feature = "val-exec-entry")]
pub fn execute<Ctx>(ctx: &mut Ctx, message: Any) -> Result<(), RouterError>
where
    Ctx: ExecutionContext,
{
    let envelope: MsgEnvelope = message.try_into()?;
    execution_handler(ctx, envelope)
}
