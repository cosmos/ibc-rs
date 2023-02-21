//! This module implements the processing logic for ICS3 (connection open
//! handshake) messages.

use crate::core::ics03_connection::connection::ConnectionEnd;
use crate::core::ics24_host::identifier::ConnectionId;

pub mod conn_open_ack;
pub mod conn_open_confirm;
pub mod conn_open_init;
pub mod conn_open_try;

/// Defines the possible states of a connection identifier in a `ConnectionResult`.
#[derive(Clone, Debug)]
pub enum ConnectionIdState {
    /// Specifies that the handler allocated a new connection identifier. This happens during the
    /// processing of either the `MsgConnectionOpenInit` or `MsgConnectionOpenTry` message.
    Generated,

    /// Specifies that the handler reused a previously-allocated connection identifier.
    Reused,
}

#[derive(Clone, Debug)]
pub struct ConnectionResult {
    /// The identifier for the connection which the handler processed. Typically this represents the
    /// newly-generated connection id (e.g., when processing `MsgConnectionOpenInit`) or
    /// an existing connection id (e.g., for `MsgConnectionOpenAck`).
    pub connection_id: ConnectionId,

    /// The state of the connection identifier (whether it was newly-generated or not).
    pub connection_id_state: ConnectionIdState,

    /// The connection end, which the handler produced as a result of processing the message.
    pub connection_end: ConnectionEnd,
}

#[cfg(test)]
pub mod test_util {
    use core::fmt::Debug;

    use crate::{core::ContextError, mock::context::MockContext, prelude::String};
    use alloc::format;

    pub enum Expect {
        Success,
        Failure(Option<ContextError>),
    }

    #[derive(Clone, Debug)]
    pub struct Fixture<M: Debug> {
        pub ctx: MockContext,
        pub msg: M,
    }

    impl<M: Debug> Fixture<M> {
        pub fn generate_error_msg(
            &self,
            expect: &Expect,
            process: &str,
            res: &Result<(), ContextError>,
        ) -> String {
            let base_error = match expect {
                Expect::Success => "step failed!",
                Expect::Failure(_) => "step passed but was supposed to fail!",
            };
            format!(
                "{process} {base_error} /n {res:?} /n {:?} /n {:?}",
                &self.msg, &self.ctx
            )
        }
    }
}
