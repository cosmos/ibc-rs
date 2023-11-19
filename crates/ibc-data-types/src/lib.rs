//! Re-exports data structures of various specifications within the Inter-Blockchain Communication (IBC) protocol.
//! Designed for universal application, enabling diverse projects across IBC ecosystem to build using a shared language.
#![no_std]
#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::disallowed_methods, clippy::disallowed_types,))]
#![deny(
    warnings,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]

pub mod primitives {
    #[doc(inline)]
    pub use ibc_primitives::*;
}

pub mod client {
    #[doc(inline)]
    pub use ibc_core_client_types::*;
}

pub mod connection {
    #[doc(inline)]
    pub use ibc_core_connection_types::*;
}

pub mod channel {
    #[doc(inline)]
    pub use ibc_core_channel_types::*;
}

pub mod commitment {
    #[doc(inline)]
    pub use ibc_core_commitment_types::*;
}

pub mod host {
    #[doc(inline)]
    pub use ibc_core_host_types::*;
}

pub mod handler {
    #[doc(inline)]
    pub use ibc_core_handler_types::*;
}

pub mod router {
    #[doc(inline)]
    pub use ibc_core_router_types::*;
}

pub mod transfer {
    #[doc(inline)]
    pub use ibc_app_transfer_types::*;
}
