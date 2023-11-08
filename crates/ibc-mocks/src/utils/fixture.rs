use core::fmt::Debug;

use ibc::core::RouterError;
use ibc::prelude::{String, *};

use crate::core::definition::MockContext;

pub enum Expect {
    Success,
    Failure(Option<RouterError>),
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
        res: &Result<(), RouterError>,
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
