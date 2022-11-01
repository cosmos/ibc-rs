use ibc_proto::google::protobuf::Any;

use crate::clients::ics07_tendermint::client_state::ClientState as TmClientState;
use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics03_connection::context::ConnectionReader;
use crate::core::ics03_connection::error::Error;
use crate::core::ics23_commitment::specs::ProofSpecs;
use crate::core::ics24_host::identifier::ChainId;

use tendermint::trust_threshold::TrustThresholdFraction as TendermintTrustThresholdFraction;

pub trait TmValidateSelfClientContext {
    fn validate_self_client(
        &self,
        ctx: &dyn ConnectionReader,
        counterparty_client_state: Any,
    ) -> Result<(), Error> {
        let counterparty_client_state = TmClientState::try_from(counterparty_client_state)
            .map_err(|_| Error::invalid_client_state())?;

        if counterparty_client_state.is_frozen() {
            return Err(Error::invalid_client_state());
        }

        let self_chain_id = self.chain_id();
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

        if self.proof_specs() != counterparty_client_state.proof_specs() {
            return Err(Error::invalid_client_state());
        }

        let _ = {
            let trust_level = counterparty_client_state.trust_level();

            TendermintTrustThresholdFraction::new(
                trust_level.numerator(),
                trust_level.denominator(),
            )
            .map_err(|_| Error::invalid_client_state())?
        };

        Ok(())
    }

    /// Returns the host chain id
    fn chain_id(&self) -> ChainId;

    /// Returns the host proof specs
    fn proof_specs(&self) -> &ProofSpecs;
}
