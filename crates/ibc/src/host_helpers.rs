use ibc_proto::google::protobuf::Any;

use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics03_connection::context::ConnectionReader;
use crate::core::ics04_channel::error::Error;

use super::client_state::ClientState as TmClientState;

/// Implementation of `ConnectionReader::validate_self_client()` for tendermint chains.
pub fn tm_validate_self_client(
    ctx: &dyn ConnectionReader,
    counterparty_client_state: Any,
) -> Result<(), Error> {
    let counterparty_client_state = TmClientState::try_from(counterparty_client_state)
        .map_err(|_| Error::invalid_client_state())?;

    if counterparty_client_state.is_frozen(){
        return Err(Error::invalid_client_state());
    }

    let self_chain_id = ctx.chain_id();
    if self_chain_id != counterparty_client_state.chain_id() {
        return Err(Error::invalid_client_state());
    }

    // counterparty client must be in the same revision as executing chain
    let self_revision_number = self_chain_id.version();
    if self_revision_number != counterparty_client_state.latest_height().revision_number() {
        return Err(Error::invalid_client_state());
    }

    if counterparty_client_state.latest_height() >= ctx.host_current_height() {
        return Err(Error::invalid_client_state());
    }

    Ok(())
}
