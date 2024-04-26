//! Defines Non-Fungible Token Transfer (ICS-721) class types.
use core::fmt::{self, Display, Error as FmtError, Formatter};
use core::str::FromStr;

use http::Uri;
pub use ibc_app_transfer_types::{TracePath, TracePrefix};
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use ibc_core::primitives::prelude::*;
#[cfg(feature = "serde")]
use ibc_core::primitives::serializers;
use ibc_proto::ibc::applications::nft_transfer::v1::ClassTrace as RawClassTrace;

use crate::data::Data;
use crate::error::NftTransferError;

/// Class ID for an NFT
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClassId(String);

impl AsRef<str> for ClassId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for ClassId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ClassId {
    type Err = NftTransferError;

    fn from_str(class_id: &str) -> Result<Self, Self::Err> {
        if class_id.trim().is_empty() {
            Err(NftTransferError::EmptyBaseClassId)
        } else {
            Ok(Self(class_id.to_string()))
        }
    }
}

/// Prefixed class to trace sources like ICS-20 PrefixedDenom
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct PrefixedClassId {
    /// A series of `{port-id}/{channel-id}`s for tracing the source of the class.
    #[cfg_attr(feature = "serde", serde(with = "serializers"))]
    #[cfg_attr(feature = "schema", schemars(with = "String"))]
    pub trace_path: TracePath,
    /// Base class of the relayed non-fungible token.
    pub base_class_id: ClassId,
}

impl PrefixedClassId {
    /// Removes the specified prefix from the trace path if there is a match, otherwise does nothing.
    pub fn remove_trace_prefix(&mut self, prefix: &TracePrefix) {
        self.trace_path.remove_prefix(prefix)
    }

    /// Adds the specified prefix to the trace path.
    pub fn add_trace_prefix(&mut self, prefix: TracePrefix) {
        self.trace_path.add_prefix(prefix)
    }
}

/// Returns true if the class ID originally came from the sender chain and false otherwise.
pub fn is_sender_chain_source(
    source_port: PortId,
    source_channel: ChannelId,
    class_id: &PrefixedClassId,
) -> bool {
    !is_receiver_chain_source(source_port, source_channel, class_id)
}

/// Returns true if the class ID originally came from the receiving chain and false otherwise.
pub fn is_receiver_chain_source(
    source_port: PortId,
    source_channel: ChannelId,
    class_id: &PrefixedClassId,
) -> bool {
    // For example, let
    // A: sender chain in this transfer, port "transfer" and channel "c2b" (to B)
    // B: receiver chain in this transfer, port "transfer" and channel "c2a" (to A)
    //
    // If B had originally sent the token in a previous transfer, then A would have stored the token as
    // "transfer/c2b/{token_denom}". Now, A is sending to B, so to check if B is the source of the token,
    // we need to check if the token starts with "transfer/c2b".
    let prefix = TracePrefix::new(source_port, source_channel);
    class_id.trace_path.starts_with(&prefix)
}

impl FromStr for PrefixedClassId {
    type Err = NftTransferError;

    /// The parsing logic is same as [`FromStr`] impl of
    /// [`PrefixedDenom`](ibc_app_transfer_types::PrefixedDenom) from ICS-20.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match TracePath::trim(s) {
            (trace_path, Some(remaining_parts)) => Ok(Self {
                trace_path,
                base_class_id: ClassId::from_str(remaining_parts)?,
            }),
            (_, None) => Ok(Self {
                trace_path: TracePath::empty(),
                base_class_id: ClassId::from_str(s)?,
            }),
        }
    }
}

impl TryFrom<RawClassTrace> for PrefixedClassId {
    type Error = NftTransferError;

    fn try_from(value: RawClassTrace) -> Result<Self, Self::Error> {
        let base_class_id = ClassId::from_str(&value.base_class_id)?;
        // FIXME: separate `TracePath` error.
        let trace_path = TracePath::from_str(&value.path)
            .map_err(|err| NftTransferError::Other(err.to_string()))?;
        Ok(Self {
            trace_path,
            base_class_id,
        })
    }
}

impl From<PrefixedClassId> for RawClassTrace {
    fn from(value: PrefixedClassId) -> Self {
        Self {
            path: value.trace_path.to_string(),
            base_class_id: value.base_class_id.to_string(),
        }
    }
}

impl From<ClassId> for PrefixedClassId {
    fn from(class_id: ClassId) -> Self {
        Self {
            trace_path: TracePath::empty(),
            base_class_id: class_id,
        }
    }
}

impl Display for PrefixedClassId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        if self.trace_path.is_empty() {
            write!(f, "{}", self.base_class_id)
        } else {
            write!(f, "{}/{}", self.trace_path, self.base_class_id)
        }
    }
}

/// Class URI for an NFT
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassUri(
    #[cfg_attr(feature = "serde", serde(with = "serializers"))]
    #[cfg_attr(feature = "schema", schemars(with = "String"))]
    Uri,
);

#[cfg(feature = "borsh")]
impl borsh::BorshSerialize for ClassUri {
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> borsh::maybestd::io::Result<()> {
        borsh::BorshSerialize::serialize(&self.to_string(), writer)
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshDeserialize for ClassUri {
    fn deserialize_reader<R: borsh::maybestd::io::Read>(
        reader: &mut R,
    ) -> borsh::maybestd::io::Result<Self> {
        let uri = String::deserialize_reader(reader)?;
        Ok(ClassUri::from_str(&uri).map_err(|_| borsh::maybestd::io::ErrorKind::Other)?)
    }
}

#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Encode for ClassUri {
    fn encode_to<T: parity_scale_codec::Output + ?Sized>(&self, writer: &mut T) {
        self.to_string().encode_to(writer);
    }
}

#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Decode for ClassUri {
    fn decode<I: parity_scale_codec::Input>(
        input: &mut I,
    ) -> Result<Self, parity_scale_codec::Error> {
        let uri = String::decode(input)?;
        ClassUri::from_str(&uri).map_err(|_| parity_scale_codec::Error::from("from str error"))
    }
}

#[cfg(feature = "parity-scale-codec")]
impl scale_info::TypeInfo for ClassUri {
    type Identity = Self;

    fn type_info() -> scale_info::Type {
        scale_info::Type::builder()
            .path(scale_info::Path::new("ClassUri", module_path!()))
            .composite(
                scale_info::build::Fields::unnamed()
                    .field(|f| f.ty::<String>().type_name("String")),
            )
    }
}

impl Display for ClassUri {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ClassUri {
    type Err = NftTransferError;

    fn from_str(class_uri: &str) -> Result<Self, Self::Err> {
        match Uri::from_str(class_uri) {
            Ok(uri) => Ok(Self(uri)),
            Err(err) => Err(NftTransferError::InvalidUri {
                uri: class_uri.to_string(),
                validation_error: err,
            }),
        }
    }
}

/// Class data for an NFT
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, derive_more::AsRef)]
pub struct ClassData(Data);

impl Display for ClassData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ClassData {
    type Err = NftTransferError;

    fn from_str(class_data: &str) -> Result<Self, Self::Err> {
        // validate the data
        let data = Data::from_str(class_data)?;
        Ok(Self(data))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("myclass")]
    #[case("transfer/channel-0/myclass")]
    #[case("transfer/channel-0/transfer/channel-1/myclass")]
    #[case("(transfer)/channel-0/myclass")]
    #[case("transfer/(channel-0)/myclass")]
    fn test_valid_class_id(#[case] class_id: &str) {
        ClassId::from_str(class_id).expect("success");
    }

    #[rstest]
    #[case("")]
    #[case("")]
    #[case("  ")]
    fn test_invalid_class_id(#[case] class_id: &str) {
        ClassId::from_str(class_id).expect_err("failure");
    }

    #[rstest]
    #[case("transfer/channel-0/myclass")]
    #[case("/myclass")]
    #[case("//myclass")]
    #[case("transfer/")]
    #[case("transfer/myclass")]
    #[case("transfer/channel-0/myclass")]
    #[case("transfer/channel-0/transfer/channel-1/myclass")]
    #[case("(transfer)/channel-0/myclass")]
    #[case("transfer/(channel-0)/myclass")]
    #[case("transfer/channel-0///")]
    fn test_valid_prefixed_class_id(#[case] class_id: &str) {
        PrefixedClassId::from_str(class_id).expect("success");
    }

    #[rstest]
    #[case("")]
    #[case("  ")]
    #[case("transfer/channel-0/")]
    #[case("transfer/channel-0/  ")]
    fn test_invalid_prefixed_class_id(#[case] class_id: &str) {
        PrefixedClassId::from_str(class_id).expect_err("failure");
    }

    #[test]
    fn test_class_id_trace() -> Result<(), NftTransferError> {
        assert_eq!(
            PrefixedClassId::from_str("transfer/channel-0/myclass")?,
            PrefixedClassId {
                trace_path: "transfer/channel-0".parse().expect("success"),
                base_class_id: "myclass".parse()?
            },
            "valid single trace info"
        );
        assert_eq!(
            PrefixedClassId::from_str("transfer/channel-0/transfer/channel-1/myclass")?,
            PrefixedClassId {
                trace_path: "transfer/channel-0/transfer/channel-1"
                    .parse()
                    .expect("success"),
                base_class_id: "myclass".parse()?
            },
            "valid multiple trace info"
        );

        Ok(())
    }

    #[test]
    fn test_class_id_serde() -> Result<(), NftTransferError> {
        let dt_str = "transfer/channel-0/myclass";
        let dt = PrefixedClassId::from_str(dt_str)?;
        assert_eq!(dt.to_string(), dt_str, "valid single trace info");

        let dt_str = "transfer/channel-0/transfer/channel-1/myclass";
        let dt = PrefixedClassId::from_str(dt_str)?;
        assert_eq!(dt.to_string(), dt_str, "valid multiple trace info");

        Ok(())
    }

    #[test]
    fn test_trace_path() -> Result<(), NftTransferError> {
        assert!(TracePath::from_str("").is_ok(), "empty trace path");
        assert!(
            TracePath::from_str("transfer/myclass").is_err(),
            "invalid trace path: bad ChannelId"
        );
        assert!(
            TracePath::from_str("transfer//myclass").is_err(),
            "malformed trace path: missing ChannelId"
        );
        assert!(
            TracePath::from_str("transfer/channel-0/").is_err(),
            "malformed trace path: trailing delimiter"
        );

        let prefix_1 = TracePrefix::new("transfer".parse().unwrap(), "channel-1".parse().unwrap());
        let prefix_2 = TracePrefix::new("transfer".parse().unwrap(), "channel-0".parse().unwrap());
        let mut trace_path = TracePath::from(vec![prefix_1.clone()]);

        trace_path.add_prefix(prefix_2.clone());
        assert_eq!(
            TracePath::from_str("transfer/channel-0/transfer/channel-1").expect("success"),
            trace_path
        );
        assert_eq!(
            TracePath::from(vec![prefix_1.clone(), prefix_2.clone()]),
            trace_path
        );

        trace_path.remove_prefix(&prefix_2);
        assert_eq!(
            TracePath::from_str("transfer/channel-1").expect("success"),
            trace_path
        );
        assert_eq!(TracePath::from(vec![prefix_1.clone()]), trace_path);

        trace_path.remove_prefix(&prefix_1);
        assert!(trace_path.is_empty());

        Ok(())
    }

    #[test]
    fn test_serde_json_roundtrip() {
        fn serde_roundtrip(class_uri: ClassUri) {
            let serialized =
                serde_json::to_string(&class_uri).expect("failed to serialize ClassUri");
            let deserialized = serde_json::from_str::<ClassUri>(&serialized)
                .expect("failed to deserialize ClassUri");

            assert_eq!(deserialized, class_uri);
        }

        let uri = "/foo/bar?baz".parse::<Uri>().unwrap();
        serde_roundtrip(ClassUri(uri));

        let uri = "https://www.rust-lang.org/install.html"
            .parse::<Uri>()
            .unwrap();
        serde_roundtrip(ClassUri(uri));
    }

    #[cfg(feature = "borsh")]
    #[test]
    fn test_borsh_roundtrip() {
        fn borsh_roundtrip(class_uri: ClassUri) {
            use borsh::{BorshDeserialize, BorshSerialize};

            let class_uri_bytes = class_uri.try_to_vec().unwrap();
            let res = ClassUri::try_from_slice(&class_uri_bytes).unwrap();

            assert_eq!(class_uri, res);
        }

        let uri = "/foo/bar?baz".parse::<Uri>().unwrap();
        borsh_roundtrip(ClassUri(uri));

        let uri = "https://www.rust-lang.org/install.html"
            .parse::<Uri>()
            .unwrap();
        borsh_roundtrip(ClassUri(uri));
    }
}
