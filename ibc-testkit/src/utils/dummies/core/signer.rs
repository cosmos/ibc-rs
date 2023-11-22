use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Signer;

pub fn dummy_account_id() -> Signer {
    "0CDA3F47EF3C4906693B170EF650EB968C5F4B2C"
        .to_string()
        .into()
}

pub fn dummy_bech32_account() -> String {
    "cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng".to_string()
}
