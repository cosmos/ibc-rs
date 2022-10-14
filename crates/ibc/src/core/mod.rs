//! The designs and logic pertaining to the transport, authentication, and
//! ordering layers of the IBC protocol.
//!
//! Naming is hard in the IBC handlers, since we deal with a client on a
//! *counterparty* chain, which is itself a light client of *self* (the chain
//! the handler is currently running on). So depending on the frame of reference
//! chosen, e.g. counterparty_client_state could mean:
//!
//! 1. the client state of the client running on the counterparty chain
//! 2. or the state of the "counterparty client" (that is, the client that we
//!    run of the counterparty chain) running on the host chain
//!
//! We remove such ambiguity by adopting the following conventions:
//! + we call "chain A" the chain that runs `OpenInit` and `OpenAck`
//! + we call "chain B" the chain that runs `OpenTry` and `OpenConfirm`
//! + In variable names,
//!     + `on_a` implies "stored on chain A"
//!     + `of_a` implies "of light client for chain A" So
//! `client_state_of_a_on_b` means "the client state of light client for chain A
//! stored on chain B"

pub mod ics02_client;
pub mod ics03_connection;
pub mod ics04_channel;
pub mod ics05_port;
pub mod ics23_commitment;
pub mod ics24_host;
pub mod ics26_routing;
