use ibc::apps::nft_transfer::context::{
    NftClassContext, NftContext, NftTransferExecutionContext, NftTransferValidationContext,
};
use ibc::apps::nft_transfer::types::{
    ClassData, ClassId, ClassUri, Memo, PrefixedClassId, TokenData, TokenId, TokenUri,
};
use ibc::core::host::types::error::HostError;
use ibc::core::host::types::identifiers::{ChannelId, PortId};
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Signer;

use super::types::{DummyNft, DummyNftClass, DummyNftTransferModule};

impl NftContext for DummyNft {
    fn get_class_id(&self) -> &ClassId {
        &self.class_id
    }

    fn get_id(&self) -> &TokenId {
        &self.token_id
    }

    fn get_uri(&self) -> Option<&TokenUri> {
        self.token_uri.as_ref()
    }

    fn get_data(&self) -> Option<&TokenData> {
        self.token_data.as_ref()
    }
}

impl NftClassContext for DummyNftClass {
    fn get_id(&self) -> &ClassId {
        &self.class_id
    }

    fn get_uri(&self) -> Option<&ClassUri> {
        self.class_uri.as_ref()
    }

    fn get_data(&self) -> Option<&ClassData> {
        self.class_data.as_ref()
    }
}

impl NftTransferValidationContext for DummyNftTransferModule {
    type AccountId = Signer;
    type Nft = DummyNft;
    type NftClass = DummyNftClass;

    fn get_port(&self) -> Result<PortId, HostError> {
        Ok(PortId::transfer())
    }

    fn can_send_nft(&self) -> Result<(), HostError> {
        Ok(())
    }

    fn can_receive_nft(&self) -> Result<(), HostError> {
        Ok(())
    }

    fn create_or_update_class_validate(
        &self,
        _class_id: &PrefixedClassId,
        _class_uri: Option<&ClassUri>,
        _class_data: Option<&ClassData>,
    ) -> Result<(), HostError> {
        Ok(())
    }

    fn escrow_nft_validate(
        &self,
        _from_account: &Self::AccountId,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _class_id: &PrefixedClassId,
        _token_id: &TokenId,
        _memo: &Memo,
    ) -> Result<(), HostError> {
        Ok(())
    }

    fn unescrow_nft_validate(
        &self,
        _to_account: &Self::AccountId,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _class_id: &PrefixedClassId,
        _token_id: &TokenId,
    ) -> Result<(), HostError> {
        Ok(())
    }

    fn mint_nft_validate(
        &self,
        _account: &Self::AccountId,
        _class_id: &PrefixedClassId,
        _token_id: &TokenId,
        _token_uri: Option<&TokenUri>,
        _token_data: Option<&TokenData>,
    ) -> Result<(), HostError> {
        Ok(())
    }

    fn burn_nft_validate(
        &self,
        _account: &Self::AccountId,
        _class_id: &PrefixedClassId,
        _token_id: &TokenId,
        _memo: &Memo,
    ) -> Result<(), HostError> {
        Ok(())
    }

    fn get_nft(
        &self,
        _class_id: &PrefixedClassId,
        _token_id: &TokenId,
    ) -> Result<Self::Nft, HostError> {
        Ok(DummyNft::default())
    }

    fn get_nft_class(&self, _class_id: &PrefixedClassId) -> Result<Self::NftClass, HostError> {
        Ok(DummyNftClass::default())
    }
}

impl NftTransferExecutionContext for DummyNftTransferModule {
    fn create_or_update_class_execute(
        &self,
        _class_id: &PrefixedClassId,
        _class_uri: Option<&ClassUri>,
        _class_data: Option<&ClassData>,
    ) -> Result<(), HostError> {
        Ok(())
    }

    fn escrow_nft_execute(
        &mut self,
        _from_account: &Self::AccountId,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _class_id: &PrefixedClassId,
        _token_id: &TokenId,
        _memo: &Memo,
    ) -> Result<(), HostError> {
        Ok(())
    }

    fn unescrow_nft_execute(
        &mut self,
        _to_account: &Self::AccountId,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _class_id: &PrefixedClassId,
        _token_id: &TokenId,
    ) -> Result<(), HostError> {
        Ok(())
    }

    fn mint_nft_execute(
        &mut self,
        _account: &Self::AccountId,
        _class_id: &PrefixedClassId,
        _token_id: &TokenId,
        _token_uri: Option<&TokenUri>,
        _token_data: Option<&TokenData>,
    ) -> Result<(), HostError> {
        Ok(())
    }

    fn burn_nft_execute(
        &mut self,
        _account: &Self::AccountId,
        _class_id: &PrefixedClassId,
        _token_id: &TokenId,
        _memo: &Memo,
    ) -> Result<(), HostError> {
        Ok(())
    }
}
