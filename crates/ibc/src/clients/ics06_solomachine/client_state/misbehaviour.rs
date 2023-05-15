use crate::prelude::*;

use crate::clients::ics06_solomachine::consensus_state::ConsensusState as SmConsensusState;
use crate::clients::ics06_solomachine::header::Header as SmHeader;
use crate::clients::ics06_solomachine::misbehaviour::Misbehaviour as SmMisbehaviour;
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
        todo!()
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
