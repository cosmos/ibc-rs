use ibc_proto::google::protobuf::Any;

use super::{
    ics26_routing::{error::RouterError, msgs::MsgEnvelope},
    ExecutionContext, ValidationContext,
};

/// Entrypoint which only performs message validation
pub fn validate<Ctx>(ctx: &Ctx, message: Any) -> Result<(), RouterError>
where
    Ctx: ValidationContext,
{
    let envelope: MsgEnvelope = message.try_into()?;
    ctx.validate(envelope)
}

/// Entrypoint which only performs message execution
pub fn execute<Ctx>(ctx: &mut Ctx, message: Any) -> Result<(), RouterError>
where
    Ctx: ExecutionContext,
{
    let envelope: MsgEnvelope = message.try_into()?;
    ctx.execute(envelope)
}
