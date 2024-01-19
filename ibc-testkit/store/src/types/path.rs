use std::fmt::{Display, Formatter};
use std::str::{from_utf8, FromStr, Utf8Error};

use displaydoc::Display as DisplayDoc;
use ibc::core::host::types::path::{Path as IbcPath, PathError};

use super::Identifier;
use crate::avl::{AsBytes, ByteSlice};

#[derive(Debug, DisplayDoc)]
pub enum Error {
    /// path isn't a valid string: `{error}`
    MalformedPathString { error: Utf8Error },
    /// parse error: `{0}`
    ParseError(String),
}

/// A new type representing a valid ICS024 `Path`.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]

pub struct Path(Vec<Identifier>);

impl Path {
    pub fn get(&self, index: usize) -> Option<&Identifier> {
        self.0.get(index)
    }
}

impl TryFrom<String> for Path {
    type Error = Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let mut identifiers = vec![];
        let parts = s.split('/'); // split will never return an empty iterator
        for part in parts {
            identifiers.push(Identifier::from(part.to_owned()));
        }
        Ok(Self(identifiers))
    }
}

impl TryFrom<&[u8]> for Path {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let s = from_utf8(value).map_err(|e| Error::MalformedPathString { error: e })?;
        s.to_owned().try_into()
    }
}

impl From<Identifier> for Path {
    fn from(id: Identifier) -> Self {
        Self(vec![id])
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|iden| iden.as_str().to_owned())
                .collect::<Vec<String>>()
                .join("/")
        )
    }
}

impl AsBytes for Path {
    fn as_bytes(&self) -> ByteSlice<'_> {
        ByteSlice::Vector(self.to_string().into_bytes())
    }
}

impl TryFrom<Path> for IbcPath {
    type Error = PathError;

    fn try_from(path: Path) -> Result<Self, Self::Error> {
        Self::from_str(path.to_string().as_str())
    }
}

impl From<IbcPath> for Path {
    fn from(ibc_path: IbcPath) -> Self {
        Self::try_from(ibc_path.to_string()).unwrap() // safety - `IbcPath`s are correct-by-construction
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(b"hello/world")]
    fn happy_test(#[case] path: &[u8]) {
        assert!(Path::try_from(path).is_ok());
    }

    // TODO(rano): add failing case for `Path::try_from`
    #[rstest]
    #[ignore]
    #[case(b"hello/@@@")]
    fn sad_test(#[case] path: &[u8]) {
        assert!(Path::try_from(path).is_err());
    }
}
