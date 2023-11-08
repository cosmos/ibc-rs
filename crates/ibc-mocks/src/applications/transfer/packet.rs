use alloc::string::ToString;

use ibc::applications::transfer::packet::PacketData;
use ibc::applications::transfer::{Memo, PrefixedCoin};
use ibc::test_utils::get_dummy_account_id;
use ibc::Signer;
use typed_builder::TypedBuilder;

/// Configuration for a `PacketData` type.
#[derive(TypedBuilder, Debug)]
#[builder(build_method(into = PacketData))]
pub struct PacketDataConfig {
    pub token: PrefixedCoin,
    #[builder(default = get_dummy_account_id())]
    pub sender: Signer,
    #[builder(default = get_dummy_account_id())]
    pub receiver: Signer,
    #[builder(default = Memo::from("".to_string()))]
    pub memo: Memo,
}

impl From<PacketDataConfig> for PacketData {
    fn from(config: PacketDataConfig) -> Self {
        PacketData {
            token: config.token,
            sender: config.sender,
            receiver: config.receiver,
            memo: config.memo,
        }
    }
}
