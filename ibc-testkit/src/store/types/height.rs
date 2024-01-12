use std::fmt::{Display, Formatter};

/// Block height
pub type RawHeight = u64;

/// Store height to query
#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum Height {
    Pending,
    Latest,
    Stable(RawHeight), // or equivalently `tendermint::block::Height`
}

impl Display for Height {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Height::Pending => write!(f, "pending"),
            Height::Latest => write!(f, "latest"),
            Height::Stable(height) => write!(f, "{}", height),
        }
    }
}

impl From<RawHeight> for Height {
    fn from(value: u64) -> Self {
        match value {
            0 => Height::Latest, // see https://docs.tendermint.com/master/spec/abci/abci.html#query
            _ => Height::Stable(value),
        }
    }
}
