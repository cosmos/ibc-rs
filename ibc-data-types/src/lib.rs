//! Re-exports data structures of various specifications within the
//! Inter-Blockchain Communication (IBC) protocol. Designed for universal
//! application, enabling diverse projects across IBC ecosystem to build using a
//! shared language.
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

/// Re-exports IBC primitive types from the `ibc-primitives` crate
pub mod primitives {
    #[doc(inline)]
    pub use ibc_primitives::*;
}

/// Re-exports data structures of all IBC core specifications
pub mod core {
    /// Re-exports ICS-02 client data structures from the
    /// `ibc-core-client-types` crate
    pub mod client {
        #[doc(inline)]
        pub use ibc_core_client_types::*;
    }
    /// Re-exports ICS-03 connection data structures from the
    /// `ibc-core-connection-types` crate
    pub mod connection {
        #[doc(inline)]
        pub use ibc_core_connection_types::*;
    }
    /// Re-exports ICS-04 channel data structures from the
    /// `ibc-core-channel-types` crate
    pub mod channel {
        #[doc(inline)]
        pub use ibc_core_channel_types::*;
    }
    /// Re-exports ICS-23 commitment data structures from the
    /// `ibc-core-commitment-types` crate
    pub mod commitment {
        #[doc(inline)]
        pub use ibc_core_commitment_types::*;
    }
    /// Re-exports ICS-24 host data structures from the `ibc-core-host-types`
    /// crate
    pub mod host {
        #[doc(inline)]
        pub use ibc_core_host_types::*;
    }
    /// Re-exports ICS-25 handler data structures from the
    /// `ibc-core-handler-types` crate
    pub mod handler {
        #[doc(inline)]
        pub use ibc_core_handler_types::*;
    }
    /// Re-exports ICS-26 routing data structures from the
    /// `ibc-core-router-types` crate
    pub mod router {
        #[doc(inline)]
        pub use ibc_core_router_types::*;
    }
}

pub mod clients {
    /// Re-exports ICS-07 tendermint client data structures from the
    /// `ibc-client-tendermint-types` crate
    pub mod tendermint {
        #[doc(inline)]
        pub use ibc_client_tendermint_types::*;
    }
    /// Re-exports ICS-08 wasm client data structures from the
    /// `ibc-client-wasm-types` crate
    pub mod wasm {
        #[doc(inline)]
        pub use ibc_client_wasm_types::*;
    }
}

/// Re-exports data structures of various IBC applications
pub mod apps {
    /// Re-exports ICS-27 client update data structures from the
    /// `ibc-core-client-types` crate
    pub mod transfer {
        #[doc(inline)]
        pub use ibc_app_transfer_types::*;
    }
}
