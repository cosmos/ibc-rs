use core::str::FromStr;

use cosmrs::AccountId;
use ibc_core::host::types::identifiers::{ConnectionId, PortId};
use ibc_core::primitives::Signer;

use crate::account::{InterchainAccount, SdkBaseAccount};
use crate::context::InterchainAccountExecutionContext;
use crate::error::InterchainAccountError;

/// Creates a new interchain account.
///
/// Generates an address using the host `ConnectionId`, the controller `PortID`,
/// and block dependent information.
/// Returns an error if an account already exists for the generated account.
/// Sets an interchain account type and updates the interchain account address mapping
pub fn create_interchain_account<Ctx>(
    ctx_b: &mut Ctx,
    conn_id_on_b: ConnectionId,
    port_id_on_a: PortId,
) -> Result<Signer, InterchainAccountError>
where
    Ctx: InterchainAccountExecutionContext,
{
    let address = ctx_b.generate_ica_address(conn_id_on_b.clone(), port_id_on_a.clone())?;

    // TODO: This is a sdk specific code. Needed a generic way to create an account
    let account = AccountId::from_str(address.as_ref()).map_err(InterchainAccountError::source)?;

    // TODO: it should be renamed to smt like `validate_account` (later PR)
    ctx_b.validate_message_signer(&address)?;

    ctx_b.get_interchain_account(&address)?;

    let interchain_account = InterchainAccount::<SdkBaseAccount>::new_with_sdk_base_account(
        account,
        port_id_on_a.clone(),
    );

    let account = ctx_b.new_interchain_account(interchain_account.clone())?;

    ctx_b.store_interchain_account(account)?;

    ctx_b.store_ica_address(conn_id_on_b, port_id_on_a, interchain_account.address())?;

    Ok(interchain_account.address())
}
