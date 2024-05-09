pub mod applications;
pub mod clients;
pub mod core;
use alloc::fmt::Debug;

use ibc::core::handler::types::error::ProtocolError;
use ibc::core::primitives::prelude::*;

use crate::testapp::ibc::core::types::DefaultIbcStore;
pub enum Expect {
    Success,
    Failure(Option<ProtocolError>),
}

#[derive(Debug)]
pub struct Fixture<M: Debug> {
    pub ctx: DefaultIbcStore,
    pub msg: M,
}

impl<M: Debug> Fixture<M> {
    pub fn generate_error_msg(
        &self,
        expect: &Expect,
        process: &str,
        res: &Result<(), ProtocolError>,
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
