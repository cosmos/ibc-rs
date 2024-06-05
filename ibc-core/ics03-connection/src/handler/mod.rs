use ibc_core_client::types::error::ClientError;
use ibc_core_handler_types::error::ContextError;
use ibc_core_host::types::identifiers::ClientId;
use ibc_primitives::proto::Any;

pub mod conn_open_ack;
pub mod conn_open_confirm;
pub mod conn_open_init;
pub mod conn_open_try;

pub(crate) fn unpack_host_client_state<CS>(
    value: Any,
    host_client_id_at_counterparty: &ClientId,
) -> Result<CS, ContextError>
where
    CS: TryFrom<Any>,
    <CS as TryFrom<Any>>::Error: Into<ClientError>,
{
    #[cfg(feature = "wasm-wrapped-client-state")]
    if host_client_id_at_counterparty.is_wasm_client_id() {
        use std::string::ToString;

        use ibc_client_wasm_types::client_state::ClientState as WasmClientState;
        use ibc_core_connection_types::error::ConnectionError;
        use prost::Message;

        let wasm_client_state = WasmClientState::try_from(value).map_err(|e| {
            ContextError::ConnectionError(ConnectionError::InvalidClientState {
                reason: e.to_string(),
            })
        })?;

        let any_client_state = <Any as Message>::decode(wasm_client_state.data.as_slice())
            .map_err(|e| {
                ContextError::ConnectionError(ConnectionError::InvalidClientState {
                    reason: e.to_string(),
                })
            })?;

        Ok(CS::try_from(any_client_state).map_err(Into::<ClientError>::into)?)
    } else {
        Ok(CS::try_from(value).map_err(Into::<ClientError>::into)?)
    }

    #[cfg(not(feature = "wasm-wrapped-client-state"))]
    {
        // this avoids lint warning for unused variable.
        let _ = host_client_id_at_counterparty;
        Ok(CS::try_from(value).map_err(Into::<ClientError>::into)?)
    }
}
