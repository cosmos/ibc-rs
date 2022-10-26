use crate::prelude::*;
use core::fmt::{Display, Error as FmtError, Formatter};
use serde_derive::{Deserialize, Serialize};

/// Type of the client, depending on the specific consensus algorithm.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClientType(&'static str);

impl ClientType {
    pub fn new(s: &'static str) -> Self {
        Self(s)
    }
    /// Yields the identifier of this client type as a string
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

impl Display for ClientType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "ClientType({})", self.as_str())
    }
}
