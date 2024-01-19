use super::Identifier;
use crate::avl::{AsBytes, ByteSlice};
use displaydoc::Display as DisplayDoc;
use ibc::core::host::types::path::{Path as IbcPath, PathError};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::str::{from_utf8, Utf8Error};

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
    use super::*;
    use std::{collections::HashSet, convert::TryFrom};

    use lazy_static::lazy_static;
    use proptest::prelude::*;
    use rand::{distributions::Standard, seq::SliceRandom};

    const ALLOWED_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                   abcdefghijklmnopqrstuvwxyz\
                                   ._+-#[]<>";

    lazy_static! {
        static ref VALID_CHARS: HashSet<char> = {
            ALLOWED_CHARS
                .iter()
                .map(|c| char::from(*c))
                .collect::<HashSet<_>>()
        };
    }

    fn gen_valid_identifier(len: usize) -> String {
        let mut rng = rand::thread_rng();

        (0..=len)
            .map(|_| {
                let idx = rng.gen_range(0..ALLOWED_CHARS.len());
                ALLOWED_CHARS[idx] as char
            })
            .collect::<String>()
    }

    fn gen_invalid_identifier(len: usize) -> String {
        let mut rng = rand::thread_rng();

        (0..=len)
            .map(|_| loop {
                let c = rng.sample::<char, _>(Standard);

                if c.is_ascii() && !VALID_CHARS.contains(&c) {
                    return c;
                }
            })
            .collect::<String>()
    }

    proptest! {
        #[test]
        fn path_with_valid_parts_is_valid(n_parts in 1usize..=10) {
            let mut rng = rand::thread_rng();

            let parts = (0..n_parts)
                .map(|_| {
                    let len = rng.gen_range(1usize..=10);
                    gen_valid_identifier(len)
                })
                .collect::<Vec<_>>();

            let path = parts.join("/");

            assert!(Path::try_from(path).is_ok());
        }

        #[test]
        #[ignore]
        fn path_with_invalid_parts_is_invalid(n_parts in 1usize..=10) {
            let mut rng = rand::thread_rng();
            let n_invalid_parts = rng.gen_range(1usize..=n_parts);
            let n_valid_parts = n_parts - n_invalid_parts;

            let mut parts = (0..n_invalid_parts)
                .map(|_| {
                    let len = rng.gen_range(1usize..=10);
                    gen_invalid_identifier(len)
                })
                .collect::<Vec<_>>();

            let mut valid_parts = (0..n_valid_parts)
                .map(|_| {
                    let len = rng.gen_range(1usize..=10);
                    gen_valid_identifier(len)
                })
                .collect::<Vec<_>>();

            parts.append(&mut valid_parts);
            parts.shuffle(&mut rng);

            let path = parts.join("/");

            assert!(Path::try_from(path).is_err());
        }
    }
}
