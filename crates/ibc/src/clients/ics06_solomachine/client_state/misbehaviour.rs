use ibc_proto::protobuf::Protobuf;

use crate::clients::ics06_solomachine::types::sign_bytes::SignBytes;
use crate::prelude::*;

use crate::clients::ics06_solomachine::client_state::SignatureAndData;
use crate::clients::ics06_solomachine::consensus_state::ConsensusState as SmConsensusState;
use crate::clients::ics06_solomachine::header::Header as SmHeader;
use crate::clients::ics06_solomachine::misbehaviour::Misbehaviour as SmMisbehaviour;
use crate::clients::ics06_solomachine::proof::verify_signature;
use crate::core::ics02_client::error::ClientError;
use crate::core::timestamp::Timestamp;
use crate::core::{ics24_host::identifier::ClientId, ValidationContext};

use super::ClientState;

impl ClientState {
    // verify_misbehaviour determines whether or not two conflicting headers at
    // the same height would have convinced the light client.
    pub fn verify_misbehaviour(
        &self,
        _ctx: &dyn ValidationContext,
        _client_id: &ClientId,
        _misbehaviour: SmMisbehaviour,
    ) -> Result<(), ClientError> {
        // NOTE: a check that the misbehaviour message data are not equal is done by
        // misbehaviour.ValidateBasic which is called by the 02-client keeper.
        // verify first signature
        self.verify_signature_and_data(_misbehaviour.clone(), _misbehaviour.signature_one.clone())
            .map_err(|_| ClientError::Other {
                description: "failed to verify signature one".into(),
            })?;

        // verify second signature
        self.verify_signature_and_data(_misbehaviour.clone(), _misbehaviour.signature_two)
            .map_err(|_| ClientError::Other {
                description: "failed to verify signature one".into(),
            })
    }

    // verifySignatureAndData verifies that the currently registered public key has signed
    // over the provided data and that the data is valid. The data is valid if it can be
    // unmarshaled into the specified data type.
    // ref: https://github.com/cosmos/ibc-go/blob/388283012124fd3cd66c9541000541d9c6767117/modules/light-clients/06-solomachine/misbehaviour_handle.go#L41
    pub fn verify_signature_and_data(
        &self,
        misbehaviour: SmMisbehaviour,
        signature_and_data: SignatureAndData,
    ) -> Result<(), ClientError> {
        let sign_bytes = SignBytes {
            sequence: misbehaviour.sequence.revision_height(),
            timestamp: signature_and_data.timestamp,
            diversifier: self.consensus_state.diversifier.clone(),
            data_type: crate::clients::ics06_solomachine::types::DataType::Header,
            data: signature_and_data.data,
        };
        let data = sign_bytes.encode_vec();

        let signature_and_data = SignatureAndData::decode_vec(&signature_and_data.signature)
            .map_err(|_| ClientError::Other {
                description: "failed to decode SignatureData".into(),
            })?;

        let public_key = self.consensus_state.public_key();

        verify_signature(public_key, data, signature_and_data).map_err(|e| ClientError::Other {
            description: e.to_string(),
        })
    }

    pub fn verify_misbehaviour_header(
        &self,
        _header: &SmHeader,
        _trusted_consensus_state: &SmConsensusState,
        _current_timestamp: Timestamp,
    ) -> Result<(), ClientError> {
        todo!()
    }

    pub fn check_for_misbehaviour_misbehavior(
        &self,
        _misbehaviour: &SmMisbehaviour,
    ) -> Result<bool, ClientError> {
        todo!()
    }
}
