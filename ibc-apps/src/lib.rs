//! Re-exports implementations and data structures of different IBC applications.
#![no_std]
#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::disallowed_methods, clippy::disallowed_types))]
#![deny(
    warnings,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]

/// Re-exports the implementation of the IBC [fungible token
/// transfer](https://github.com/cosmos/ibc/blob/main/spec/app/ics-020-fungible-token-transfer/README.md)
/// (ICS-20) application logic.
pub mod transfer {
    #[doc(inline)]
    pub use ibc_app_transfer::*;
}

/// Re-exports the implementation of the IBC [Non-Fungible Token
/// Transfer](https://github.com/cosmos/ibc/blob/main/spec/app/ics-721-nft-transfer/README.md)
/// (ICS-721) application logic.
pub mod nft_transfer {
    #[doc(inline)]
    #[cfg(feature = "nft-transfer")]
    pub use ibc_app_nft_transfer::*;
}
