use std::error::Error;

use cosmwasm_std::StdError;
use derive_more::{Display, From};
use ibc_core::client::types::error::ClientError;
use ibc_core::commitment_types::error::CommitmentError;
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::error::IdentifierError;
use ibc_core::host::types::path::PathError;

#[derive(From, Display, Debug)]
pub enum ContractError {
    Std(StdError),
    #[display(fmt = "invalid message: {_0}")]
    InvalidMsg(String),
    #[display(fmt = "IBC validation/execution context error: {_0}")]
    Context(ContextError),
    #[display(fmt = "IBC 02-client error: {_0}")]
    Ics02ClientError(ClientError),
    #[display(fmt = "IBC commitment error: {_0}")]
    Commitment(CommitmentError),
    #[display(fmt = "IBC identifier error: {_0}")]
    Identifier(IdentifierError),
    #[display(fmt = "IBC path error: {_0}")]
    Path(PathError),
    #[display(fmt = "Proto decode error: {_0}")]
    ProtoDecode(prost::DecodeError),
}

impl Error for ContractError {}

impl From<ContractError> for StdError {
    fn from(err: ContractError) -> StdError {
        StdError::generic_err(err.to_string())
    }
}
