//! Implementation of the [fungible token transfer module](https://github.com/cosmos/ibc/blob/main/spec/app/ics-020-fungible-token-transfer/README.md) (ICS-20)

pub mod acknowledgement;
pub mod amount;
pub mod coin;
pub mod context;
pub mod denom;
pub mod error;
pub mod events;
pub mod memo;
pub mod msgs;
pub mod packet;

pub use amount::*;
pub use coin::*;
pub use denom::*;
pub use memo::*;

/// Module identifier for the ICS20 application.
pub const MODULE_ID_STR: &str = "transfer";

/// The port identifier that the ICS20 applications
/// typically bind with.
pub const PORT_ID_STR: &str = "transfer";

/// ICS20 application current version.
pub const VERSION: &str = "ics20-1";

mod relay;

pub use relay::send_transfer::{send_transfer, send_transfer_execute, send_transfer_validate};
