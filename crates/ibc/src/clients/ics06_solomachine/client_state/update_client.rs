use crate::prelude::*;

use crate::clients::ics06_solomachine::header::Header as SmHeader;
use crate::core::ics02_client::error::ClientError;
use crate::core::{ics24_host::identifier::ClientId, ValidationContext};

use super::ClientState;

impl ClientState {
    pub fn verify_header(
        &self,
        _ctx: &dyn ValidationContext,
        _client_id: &ClientId,
        _header: SmHeader,
    ) -> Result<(), ClientError> {
        todo!()
    }

    pub fn check_for_misbehaviour_update_client(
        &self,
        _ctx: &dyn ValidationContext,
        _client_id: &ClientId,
        _header: SmHeader,
    ) -> Result<bool, ClientError> {
        todo!()
    }
}
