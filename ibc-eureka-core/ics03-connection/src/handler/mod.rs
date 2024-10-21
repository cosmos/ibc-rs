use ibc_eureka_core_client::types::error::ClientError;
use ibc_eureka_core_connection_types::error::ConnectionError;
#[cfg(feature = "wasm-client")]
use ibc_eureka_core_host::types::error::DecodingError;
use ibc_eureka_core_host::types::identifiers::ClientId;
use ibc_primitives::proto::Any;

pub mod conn_open_ack;
pub mod conn_open_confirm;
pub mod conn_open_init;
pub mod conn_open_try;

/// Unpacks the client state from the format that is stored at the counterparty chain.
///
/// Currently, the IBC-go enabled chains stores Wasm LightClient states in a WasmClientState
/// wrapper. This function unpacks the client state from the WasmClientState wrapper
/// if the client identifier at counterparty is of Wasm client type.
pub(crate) fn unpack_host_client_state<CS>(
    value: Any,
    host_client_id_at_counterparty: &ClientId,
) -> Result<CS, ConnectionError>
where
    CS: TryFrom<Any>,
    <CS as TryFrom<Any>>::Error: Into<ClientError>,
{
    #[cfg(feature = "wasm-client")]
    if host_client_id_at_counterparty.is_wasm_client_id() {
        use ibc_client_wasm_types::client_state::ClientState as WasmClientState;
        use prost::Message;

        let wasm_client_state =
            WasmClientState::try_from(value).expect("TODO(rano): propagate the error");

        let any_client_state = <Any as Message>::decode(wasm_client_state.data.as_slice())
            .map_err(|e| ConnectionError::Decoding(DecodingError::Prost(e)))?;

        Ok(CS::try_from(any_client_state).map_err(Into::<ClientError>::into)?)
    } else {
        Ok(CS::try_from(value).map_err(Into::<ClientError>::into)?)
    }

    #[cfg(not(feature = "wasm-client"))]
    {
        // this avoids lint warning for unused variable.
        let _ = host_client_id_at_counterparty;
        Ok(CS::try_from(value).map_err(Into::<ClientError>::into)?)
    }
}

/// Pack the host consensus state in the expected format stored at the counterparty chain.
///
/// Currently, the IBC-go enabled chains stores Wasm LightClient states in a WasmConsensusState
/// wrapper. This function packs the consensus state in the WasmConsensusState wrapper
/// if the client identifier at counterparty is of Wasm client type.
pub(crate) fn pack_host_consensus_state<CS>(
    value: CS,
    host_client_id_at_counterparty: &ClientId,
) -> Any
where
    CS: Into<Any>,
{
    let any_value = value.into();

    #[cfg(feature = "wasm-client")]
    if host_client_id_at_counterparty.is_wasm_client_id() {
        use ibc_client_wasm_types::consensus_state::ConsensusState as WasmConsensusState;
        use prost::Message;

        let wasm_consensus_state = WasmConsensusState::new(any_value.encode_to_vec());

        wasm_consensus_state.into()
    } else {
        any_value
    }

    #[cfg(not(feature = "wasm-client"))]
    {
        // this avoids lint warning for unused variable.
        let _ = host_client_id_at_counterparty;
        any_value
    }
}
