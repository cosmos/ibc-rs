use ibc_proto::google::protobuf::Any;

use super::{ics26_routing::{msgs::MsgEnvelope, error::RouterError}, ValidationContext};

/// Entrypoint which only performs message validation
pub fn validate<Ctx>(ctx: &Ctx, message: Any) -> Result<(), RouterError>
where
    Ctx: ValidationContext,
{
    let envelope: MsgEnvelope = message.try_into()?;
    ctx.validate(envelope)
}
