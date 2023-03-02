//! This module implements the processing logic for ICS3 (connection open
//! handshake) messages.

pub mod conn_open_ack;
pub mod conn_open_confirm;
pub mod conn_open_init;
pub mod conn_open_try;

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
