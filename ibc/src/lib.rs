#![no_std]
#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::disallowed_methods, clippy::disallowed_types))]
#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]
//! This library re-exports implementations of all the Inter-Blockchain
//! Communication (IBC) specifications available in [`ibc-rs`][ibc-rs]
//! repository. IBC is a distributed protocol that enables communication between
//! distinct sovereign blockchains.
//!
//! The layout of this crate mirrors the organization of the [IBC
//! Standard][ibc-standard]:
//!
//! + [Core](core) implements the transport, authentication, and ordering layers
//!   of the IBC protocol.
//!
//! + [Clients](clients) consists of implementations of client verification
//! algorithms (following the base client interface that is defined in `Core`)
//! for specific consensus algorithms. A chain uses these verification
//! algorithms to verify the state of remote chains.
//!
//! + [Applications](apps) consists of implementations of some IBC applications.
//! This is the part of the protocol that abstracts away the core protocol and
//! focuses solely on business logic.
//!
//! [ibc-standard]: https://github.com/cosmos/ibc
//! [ibc-rs]: https://github.com/cosmos/ibc-rs

#[cfg(any(test, feature = "std"))]
extern crate std;

/// Re-exports primitive types and traits from the `ibc-primitives` crate.
pub mod primitives {
    pub use ibc_primitives::*;
}

/// Re-exports implementations of all the IBC core (TAO) modules.
pub mod core {
    #[doc(inline)]
    pub use ibc_core::*;
}

/// Re-exports implementations of IBC light clients.
pub mod clients {
    #[doc(inline)]
    pub use ibc_clients::*;
}

/// Re-exports implementations of various IBC applications.
pub mod apps {
    #[doc(inline)]
    pub use ibc_apps::*;
}

/// Re-exports Cosmos-specific utility types, traits, and implementations.
pub mod cosmos_host {
    pub use ibc_core_host_cosmos::*;
}

/// Re-exports convenient derive macros from `ibc-derive` crate.
pub mod derive {
    /// A derive macro for implementing the
    /// [`ClientState`](crate::core::client::context::client_state::ClientState)
    /// trait for enums. Enums with variants that also implement the
    /// [`ClientState`](crate::core::client::context::client_state::ClientState)
    /// trait can leverage this macro for automatic implementation.
    ///
    /// To specify the generic arguments for `ClientState`, use the following
    /// attributes:
    ///
    /// - `#[validation(<YourClientValidationContext>)]`
    /// - `#[execution(<YourClientExecutionContext>)]`
    ///
    /// The argument to the `validation` or `execution` attributes may contain
    /// lifetimes or generic types and even that types might be bounded by
    /// traits. For instance:
    ///
    /// - `#[validation(Context<S>)]`
    /// - `#[validation(Context<'a, S>)]`
    /// - `#[validation(Context<'a, S: Clone>)]`
    pub use ibc_derive::IbcClientState as ClientState;
    /// A derive macro for implementing the
    /// [`ConsensusState`](crate::core::client::context::consensus_state::ConsensusState)
    /// trait for enums. Enums with variants that also implement the
    /// [`ConsensusState`](crate::core::client::context::consensus_state::ConsensusState)
    /// trait can leverage this macro for automatic implementation..
    pub use ibc_derive::IbcConsensusState as ConsensusState;
}
