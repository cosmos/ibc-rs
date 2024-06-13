use ibc_core::host::types::identifiers::{ChannelId, ConnectionId, PortId};
use ibc_core::host::{ExecutionContext, ValidationContext};
use ibc_core::primitives::Signer;

use super::account::{BaseAccount, InterchainAccount};
use super::error::InterchainAccountError;
use super::host::params::Params;

pub trait InterchainAccountValidationContext: ValidationContext {
    type AccountId: TryFrom<Signer>;

    /// Returns true if the controller functionality is enabled on the chain
    fn is_controller_enabled(&self) -> bool;

    /// Returns the active `ChannelId` from the store by the provided
    /// `ConnectionId` and `PortId`
    fn get_active_channel_id(
        &self,
        connection_id: &ConnectionId,
        port_id: &PortId,
    ) -> Result<ChannelId, InterchainAccountError>;

    /// Returns the parameters needed for functioning as a host chain
    fn get_params(&self) -> Result<Params, InterchainAccountError>;

    /// Returns the `AccountId` for the given address
    fn get_interchain_account(
        &self,
        address: &Signer,
    ) -> Result<Self::AccountId, InterchainAccountError>;

    /// Returns the InterchainAccount address from the store associated with
    /// the provided ConnectionId and PortId
    fn get_ica_address(
        &self,
        connection_id: &ConnectionId,
        port_id: &PortId,
    ) -> Result<Signer, InterchainAccountError>;
}

pub trait InterchainAccountExecutionContext:
    ExecutionContext + InterchainAccountValidationContext
{
    /// Stores the active `ChannelId` to the store by the provided
    /// `ConnectionId` and `PortId`
    fn store_active_channel_id(
        &mut self,
        connection_id: ConnectionId,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> Result<(), InterchainAccountError>;

    /// Stores the parameters for functioning as a host chain
    fn store_params(&mut self, params: Params) -> Result<(), InterchainAccountError>;

    /// Generates a new interchain account address.
    ///
    /// It uses the host `ConnectionId`, the controller `PortId`, and may also
    /// (in case of Cosmos SDK chains) incorporate block dependent information.
    fn generate_ica_address(
        &self,
        connection_id: ConnectionId,
        port_id: PortId,
    ) -> Result<Signer, InterchainAccountError>;

    /// Stores the interchain account address
    fn store_ica_address(
        &mut self,
        connection_id: ConnectionId,
        port_id: PortId,
        interchain_account_address: Signer,
    ) -> Result<(), InterchainAccountError>;

    /// Creates a new interchain account with the provided account information
    fn new_interchain_account<A: BaseAccount>(
        &mut self,
        account: InterchainAccount<A>,
    ) -> Result<Self::AccountId, InterchainAccountError>;

    /// Stores the created interchain account to the store
    fn store_interchain_account(
        &mut self,
        account: Self::AccountId,
    ) -> Result<(), InterchainAccountError>;
}
