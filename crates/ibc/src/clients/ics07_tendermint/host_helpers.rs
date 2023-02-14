use alloc::format;
use alloc::string::ToString;
use ibc_proto::google::protobuf::Any;

use crate::clients::ics07_tendermint::client_state::ClientState as TmClientState;
use crate::core::context::HostChainContext;
use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics24_host::error::HostError;

use tendermint::trust_threshold::TrustThresholdFraction as TendermintTrustThresholdFraction;

/// Provides an implementation of `validate_self_client` for Tendermint-based
/// hosts.
pub fn validate_self_tendermint_client<Ctx>(
    ctx: &Ctx,
    counterparty_client_state: Any,
) -> Result<(), HostError>
where
    Ctx: HostChainContext,
{
    let counterparty_client_state =
        TmClientState::try_from(counterparty_client_state).map_err(|_| {
            HostError::InvalidSelfClientState {
                reason: "client must be a tendermint client".to_string(),
            }
        })?;

    if counterparty_client_state.is_frozen() {
        return Err(HostError::InvalidSelfClientState {
            reason: "client is frozen".to_string(),
        });
    }

    let self_chain_id = ctx.chain_id();
    if self_chain_id != &counterparty_client_state.chain_id {
        return Err(HostError::InvalidSelfClientState {
            reason: format!(
                "invalid chain-id. expected: {}, got: {}",
                self_chain_id, counterparty_client_state.chain_id
            ),
        });
    }

    let self_revision_number = self_chain_id.version();
    if self_revision_number != counterparty_client_state.latest_height().revision_number() {
        return Err(HostError::InvalidSelfClientState {
            reason: format!(
                "client is not in the same revision as the chain. expected: {}, got: {}",
                self_revision_number,
                counterparty_client_state.latest_height().revision_number()
            ),
        });
    }

    let host_current_height = ctx.host_height().map_err(|_| HostError::MissingHeight)?;

    if counterparty_client_state.latest_height() >= host_current_height {
        return Err(HostError::InvalidSelfClientState {
            reason: format!(
                "client has latest height {} greater than or equal to chain height {}",
                counterparty_client_state.latest_height(),
                host_current_height
            ),
        });
    }

    if ctx.proof_specs() != &counterparty_client_state.proof_specs {
        return Err(HostError::InvalidSelfClientState {
            reason: format!(
                "client has invalid proof specs. expected: {:?}, got: {:?}",
                ctx.proof_specs(),
                counterparty_client_state.proof_specs
            ),
        });
    }

    let trust_level = counterparty_client_state.trust_level;
    TendermintTrustThresholdFraction::new(trust_level.numerator(), trust_level.denominator())
        .map_err(|_| HostError::InvalidSelfClientState {
            reason: "invalid trust level".to_string(),
        })?;

    match ctx.unbonding_period() {
        Some(host_unbonding_period) => {
            if host_unbonding_period != counterparty_client_state.unbonding_period {
                return Err(HostError::InvalidSelfClientState {
                    reason: format!(
                        "unbonding period must be greater than trusting period. unbonding period ({:?}) < trusting period ({:?})",
                        host_unbonding_period,
                        counterparty_client_state.trusting_period
                    ),
                });
            }
        }
        None => {
            return Err(HostError::InvalidSelfClientState {
                reason: "unbonding period must be set".to_string(),
            });
        }
    }

    if counterparty_client_state.unbonding_period < counterparty_client_state.trusting_period {
        return Err(HostError::InvalidSelfClientState{ reason: format!(
                "unbonding period must be greater than trusting period. unbonding period ({:?}) < trusting period ({:?})",
                counterparty_client_state.unbonding_period,
                counterparty_client_state.trusting_period
            )});
    }

    if !counterparty_client_state.upgrade_path.is_empty()
        && ctx.upgrade_path() != counterparty_client_state.upgrade_path
    {
        return Err(HostError::InvalidSelfClientState {
            reason: format!(
                "invalid upgrade path. expected: {:?}, got: {:?}",
                ctx.upgrade_path(),
                counterparty_client_state.upgrade_path
            ),
        });
    }
    Ok(())
}
