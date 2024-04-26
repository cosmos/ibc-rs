//! Defines types to represent "denominations" [as defined in ICS-20](https://github.com/cosmos/ibc/blob/main/spec/app/ics-020-fungible-token-transfer/README.md#data-structures)
use core::fmt::{Display, Error as FmtError, Formatter};
use core::str::FromStr;

use derive_more::{Display, From};
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use ibc_core::primitives::prelude::*;
#[cfg(feature = "serde")]
use ibc_core::primitives::serializers;
use ibc_proto::ibc::applications::transfer::v1::DenomTrace as RawDenomTrace;

use super::error::TokenTransferError;

/// The "base" of a denomination.
///
/// For example, given the token `my_port-1/my_channel-1/my_port-2/my_channel-2/base_denom`,
/// `base_denom` is the "base" of the denomination
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Display)]
pub struct BaseDenom(String);

impl BaseDenom {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for BaseDenom {
    type Err = TokenTransferError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            Err(TokenTransferError::EmptyBaseDenom)
        } else {
            Ok(BaseDenom(s.to_owned()))
        }
    }
}

/// One hop in a token's trace, which consists of the port and channel IDs of the sender
///
/// For example, given the token `my_port-1/my_channel-1/my_port-2/my_channel-2/base_denom`,
/// `my_port-1/my_channel-1` is a trace prefix, and `my_port-2/my_channel-2` is another one.
/// See [TracePath] which stitches trace prefixes together.
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
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct TracePrefix {
    port_id: PortId,
    channel_id: ChannelId,
}

impl TracePrefix {
    pub fn new(port_id: PortId, channel_id: ChannelId) -> Self {
        Self {
            port_id,
            channel_id,
        }
    }

    /// Returns a string slice with [`TracePrefix`] removed.
    ///
    /// If the string starts with a [`TracePrefix`], i.e. `{port-id}/channel-{id}`,
    /// it returns a tuple of the removed [`TracePrefix`] and the substring after the prefix.
    ///
    /// If the substring is empty, it returns `None`.
    /// Otherwise the substring starts with `/`. In that case,
    /// the leading `/` is stripped and returned.
    ///
    /// If the string does not start with a [`TracePrefix`], this method returns `None`.
    ///
    /// This method is analogous to `strip_prefix` from the standard library.
    pub fn strip(s: &str) -> Option<(Self, Option<&str>)> {
        // The below two chained `split_once` calls emulate a virtual `split_twice` call,
        // which is not available in the standard library.
        let (port_id_s, remaining) = s.split_once('/')?;
        let (channel_id_s, remaining) = remaining
            .split_once('/')
            .map(|(a, b)| (a, Some(b)))
            .unwrap_or_else(|| (remaining, None));

        let port_id = port_id_s.parse().ok()?;
        let channel_id = channel_id_s.parse().ok()?;

        Some((Self::new(port_id, channel_id), remaining))
    }
}

impl Display for TracePrefix {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}/{}", self.port_id, self.channel_id)
    }
}

/// A full trace path modelled as a collection of `TracePrefix`s.
///
/// Internally, the `TracePath` is modelled as a `Vec<TracePrefix>` but with the order reversed, i.e.
/// "transfer/channel-0/transfer/channel-1/uatom" => `["transfer/channel-1", "transfer/channel-0"]`
/// This is done for ease of addition/removal of prefixes.
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
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, From)]
pub struct TracePath(Vec<TracePrefix>);

impl TracePath {
    /// Returns true iff this path starts with the specified prefix
    pub fn starts_with(&self, prefix: &TracePrefix) -> bool {
        self.0.last().map(|p| p == prefix).unwrap_or(false)
    }

    /// Removes the specified prefix from the path if there is a match, otherwise does nothing.
    pub fn remove_prefix(&mut self, prefix: &TracePrefix) {
        if self.starts_with(prefix) {
            self.0.pop();
        }
    }

    /// Adds the specified prefix to the path.
    pub fn add_prefix(&mut self, prefix: TracePrefix) {
        self.0.push(prefix)
    }

    /// Returns true if the path is empty and false otherwise.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return empty trace path
    pub fn empty() -> Self {
        Self(vec![])
    }

    /// Returns a string slice with [`TracePath`] or all [`TracePrefix`]es repeatedly removed.
    ///
    /// If the string starts with a [`TracePath`], it returns a tuple of the removed
    /// [`TracePath`] and the substring after the [`TracePath`].
    ///
    /// If the substring is empty, it returns `None`.
    /// Otherwise the substring starts with `/`. In that case,
    /// the leading `/` is stripped and returned.
    ///
    /// If the string does not contain any [`TracePrefix`], it returns the original string.
    ///
    /// This method is analogous to `trim_start_matches` from the standard library.
    pub fn trim(s: &str) -> (Self, Option<&str>) {
        // We can't use `TracePrefix::empty()` with `TracePrefix::add_prefix()`.
        // Because we are stripping prefixes in reverse order.
        let mut trace_prefixes = vec![];
        let mut current_remaining_opt = Some(s);

        loop {
            let Some(current_remaining_s) = current_remaining_opt else {
                break;
            };

            let Some((trace_prefix, next_remaining_opt)) = TracePrefix::strip(current_remaining_s)
            else {
                break;
            };

            trace_prefixes.push(trace_prefix);
            current_remaining_opt = next_remaining_opt;
        }

        // Reversing is needed, as [`TracePath`] requires quick addition/removal
        // of prefixes which is more performant from the end of a [`Vec`].
        trace_prefixes.reverse();
        (Self(trace_prefixes), current_remaining_opt)
    }
}

impl FromStr for TracePath {
    type Err = TokenTransferError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(TracePath::empty());
        }

        let (trace_path, remaining_parts) = TracePath::trim(s);
        remaining_parts
            .is_none()
            .then_some(trace_path)
            .ok_or_else(|| TokenTransferError::MalformedTrace(s.to_string()))
    }
}

impl Display for TracePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let path = self
            .0
            .iter()
            .rev()
            .map(|prefix| prefix.to_string())
            .collect::<Vec<String>>()
            .join("/");
        write!(f, "{path}")
    }
}

/// A type that contains the base denomination for ICS20 and the source tracing information path.
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
pub struct PrefixedDenom {
    /// A series of `{port-id}/{channel-id}`s for tracing the source of the token.
    #[cfg_attr(feature = "serde", serde(with = "serializers"))]
    #[cfg_attr(feature = "schema", schemars(with = "String"))]
    pub trace_path: TracePath,
    /// Base denomination of the relayed fungible token.
    pub base_denom: BaseDenom,
}

impl PrefixedDenom {
    /// Removes the specified prefix from the trace path if there is a match, otherwise does nothing.
    pub fn remove_trace_prefix(&mut self, prefix: &TracePrefix) {
        self.trace_path.remove_prefix(prefix)
    }

    /// Adds the specified prefix to the trace path.
    pub fn add_trace_prefix(&mut self, prefix: TracePrefix) {
        self.trace_path.add_prefix(prefix)
    }
}

/// Returns true if the denomination originally came from the sender chain and
/// false otherwise.
///
/// Note: It is better to think of the "source" chain as the chain that
/// escrows/unescrows the token, while the other chain mints/burns the tokens,
/// respectively. A chain being the "source" of a token does NOT mean it is the
/// original creator of the token (e.g. "uatom"), as "source" might suggest.
///
/// This means that in any given transfer, a chain can very well be the source
/// of a token of which it is not the creator. For example, let
///
/// A: sender chain in this transfer, port "transfer" and channel "c2b" (to B)
/// B: receiver chain in this transfer, port "transfer" and channel "c2a" (to A)
/// token denom: "transfer/someOtherChannel/someDenom"
///
/// A, initiator of the transfer, needs to figure out if it should escrow the
/// tokens, or burn them. If B had originally sent the token to A in a previous
/// transfer, then A would have stored the token as "transfer/c2b/someDenom".
/// Now, A is sending to B, so to check if B is the source of the token, we need
/// to check if the token starts with "transfer/c2b". In this example, it
/// doesn't, so the token doesn't originate from B. A is considered the source,
/// even though it is not the creator of the token. Specifically, the token was
/// created by the chain at the other end of A's port "transfer" and channel
/// "someOtherChannel".
pub fn is_sender_chain_source(
    source_port: PortId,
    source_channel: ChannelId,
    denom: &PrefixedDenom,
) -> bool {
    !is_receiver_chain_source(source_port, source_channel, denom)
}

/// Returns true if the denomination originally came from the receiving chain and false otherwise.
pub fn is_receiver_chain_source(
    source_port: PortId,
    source_channel: ChannelId,
    denom: &PrefixedDenom,
) -> bool {
    // For example, let
    // A: sender chain in this transfer, port "transfer" and channel "c2b" (to B)
    // B: receiver chain in this transfer, port "transfer" and channel "c2a" (to A)
    //
    // If B had originally sent the token in a previous transfer, then A would have stored the token as
    // "transfer/c2b/{token_denom}". Now, A is sending to B, so to check if B is the source of the token,
    // we need to check if the token starts with "transfer/c2b".
    let prefix = TracePrefix::new(source_port, source_channel);
    denom.trace_path.starts_with(&prefix)
}

impl FromStr for PrefixedDenom {
    type Err = TokenTransferError;

    /// Initializes a [`PrefixedDenom`] from a string that adheres to the format
    /// `{nth-port-id/channel-<index>}/{(n-1)th-port-id/channel-<index>}/.../{1st-port-id/channel-<index>}/<base_denom>`.
    /// A [`PrefixedDenom`] exhibits a sequence of `{ith-port-id/channel-<index>}` pairs.
    /// This sequence makes up the [`TracePath`] of the [`PrefixedDenom`].
    ///
    /// This [`PrefixedDenom::from_str`] implementation _left-split-twice_ the argument string
    /// using `/` delimiter. Then it peeks into the first two segments and attempts to convert
    /// the first segment into a [`PortId`] and the second into a [`ChannelId`].
    /// This continues on the third remaining segment in a loop until a
    /// `{port-id/channel-id}` pair cannot be created from the top two segments.
    /// The remaining parts of the string are then considered the [`BaseDenom`].
    ///
    /// For example, given the following denom trace:
    /// "transfer/channel-75/factory/stars16da2uus9zrsy83h23ur42v3lglg5rmyrpqnju4/dust",
    /// the first two `/`-delimited segments are `"transfer"` and `"channel-75"`. The
    /// first is a valid [`PortId`], and the second is a valid [`ChannelId`], so that becomes
    /// the first `{port-id/channel-id}` pair that gets added as part of the [`TracePath`]
    /// of the [`PrefixedDenom`]. The next two segments are `"factory"`, a
    /// valid [`PortId`], and `"stars16da2uus9zrsy83h23ur42v3lglg5rmyrpqnju4"`, an invalid [`ChannelId`].
    /// The loop breaks at this point, resulting in a [`TracePath`] of `"transfer/channel-75"`
    /// and a [`BaseDenom`] of `"factory/stars16da2uus9zrsy83h23ur42v3lglg5rmyrpqnju4/dust"`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match TracePath::trim(s) {
            (trace_path, Some(remaining_parts)) => Ok(Self {
                trace_path,
                base_denom: BaseDenom::from_str(remaining_parts)?,
            }),
            (_, None) => Ok(Self {
                trace_path: TracePath::empty(),
                base_denom: BaseDenom::from_str(s)?,
            }),
        }
    }
}

impl TryFrom<RawDenomTrace> for PrefixedDenom {
    type Error = TokenTransferError;

    fn try_from(value: RawDenomTrace) -> Result<Self, Self::Error> {
        let base_denom = BaseDenom::from_str(&value.base_denom)?;
        let trace_path = TracePath::from_str(&value.path)?;
        Ok(Self {
            trace_path,
            base_denom,
        })
    }
}

impl From<PrefixedDenom> for RawDenomTrace {
    fn from(value: PrefixedDenom) -> Self {
        Self {
            path: value.trace_path.to_string(),
            base_denom: value.base_denom.to_string(),
        }
    }
}

impl From<BaseDenom> for PrefixedDenom {
    fn from(denom: BaseDenom) -> Self {
        Self {
            trace_path: TracePath::empty(),
            base_denom: denom,
        }
    }
}

impl Display for PrefixedDenom {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        if self.trace_path.0.is_empty() {
            write!(f, "{}", self.base_denom)
        } else {
            write!(f, "{}/{}", self.trace_path, self.base_denom)
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("transfer")]
    #[case("transfer/channel-1/ica")]
    fn test_invalid_raw_demon_trace_parsing(#[case] trace_path: &str) {
        let raw_denom_trace = RawDenomTrace {
            path: trace_path.to_string(),
            base_denom: "uatom".to_string(),
        };

        PrefixedDenom::try_from(raw_denom_trace).expect_err("failure");
    }

    #[rstest]
    #[case("uatom")]
    #[case("atom")]
    fn test_accepted_denom(#[case] denom_str: &str) {
        BaseDenom::from_str(denom_str).expect("success");
    }

    #[rstest]
    #[case("")]
    #[case(" ")]
    fn test_rejected_denom(#[case] denom_str: &str) {
        BaseDenom::from_str(denom_str).expect_err("failure");
    }

    #[rstest]
    #[case(
        "transfer/channel-75",
        "factory/stars16da2uus9zrsy83h23ur42v3lglg5rmyrpqnju4/dust"
    )]
    #[case(
        "transfer/channel-75/transfer/channel-123/transfer/channel-1023/transfer/channel-0",
        "factory/stars16da2uus9zrsy83h23ur42v3lglg5rmyrpqnju4/dust"
    )]
    #[case(
        "transfer/channel-75/transfer/channel-123/transfer/channel-1023/transfer/channel-0",
        "//////////////////////dust"
    )]
    #[case("transfer/channel-0", "uatom")]
    #[case("transfer/channel-0/transfer/channel-1", "uatom")]
    #[case("", "/")]
    #[case("", "transfer/uatom")]
    #[case("", "transfer/atom")]
    #[case("", "transfer//uatom")]
    #[case("", "/uatom")]
    #[case("", "//uatom")]
    #[case("", "transfer/")]
    #[case("", "(transfer)/channel-0/uatom")]
    #[case("", "transfer/(channel-0)/uatom")]
    // https://github.com/cosmos/ibc-go/blob/e2ad31975f2ede592912b86346b5ebf055c9e05f/modules/apps/transfer/types/trace_test.go#L17-L38
    #[case("", "uatom")]
    #[case("", "uatom/")]
    #[case("", "gamm/pool/1")]
    #[case("", "gamm//pool//1")]
    #[case("transfer/channel-1", "uatom")]
    #[case("customtransfer/channel-1", "uatom")]
    #[case("transfer/channel-1", "uatom/")]
    #[case(
        "transfer/channel-1",
        "erc20/0x85bcBCd7e79Ec36f4fBBDc54F90C643d921151AA"
    )]
    #[case("transfer/channel-1", "gamm/pool/1")]
    #[case("transfer/channel-1", "gamm//pool//1")]
    #[case("transfer/channel-1/transfer/channel-2", "uatom")]
    #[case("customtransfer/channel-1/alternativetransfer/channel-2", "uatom")]
    #[case("", "transfer/uatom")]
    #[case("", "transfer//uatom")]
    #[case("", "channel-1/transfer/uatom")]
    #[case("", "uatom/transfer")]
    #[case("", "transfer/channel-1")]
    #[case("transfer/channel-1", "transfer")]
    #[case("", "transfer/channelToA/uatom")]
    fn test_strange_but_accepted_prefixed_denom(
        #[case] prefix: &str,
        #[case] denom: &str,
    ) -> Result<(), TokenTransferError> {
        let pd_s = if prefix.is_empty() {
            denom.to_owned()
        } else {
            format!("{prefix}/{denom}")
        };
        let pd = PrefixedDenom::from_str(&pd_s)?;

        assert_eq!(pd.to_string(), pd_s);
        assert_eq!(pd.trace_path.to_string(), prefix);
        assert_eq!(pd.base_denom.to_string(), denom);

        Ok(())
    }

    #[rstest]
    #[case("")]
    #[case("   ")]
    #[case("transfer/channel-1/")]
    #[case("transfer/channel-1/transfer/channel-2/")]
    #[case("transfer/channel-21/transfer/channel-23/  ")]
    #[case("transfer/channel-0/")]
    #[should_panic(expected = "EmptyBaseDenom")]
    fn test_prefixed_empty_base_denom(#[case] pd_s: &str) {
        PrefixedDenom::from_str(pd_s).expect("error");
    }

    #[rstest]
    fn test_trace_path_order() {
        let mut prefixed_denom =
            PrefixedDenom::from_str("customtransfer/channel-1/alternativetransfer/channel-2/uatom")
                .expect("no error");

        assert_eq!(
            prefixed_denom.trace_path.to_string(),
            "customtransfer/channel-1/alternativetransfer/channel-2"
        );
        assert_eq!(prefixed_denom.base_denom.to_string(), "uatom");

        let trace_prefix_1 = TracePrefix::new(
            "alternativetransfer".parse().unwrap(),
            "channel-2".parse().unwrap(),
        );

        let trace_prefix_2 = TracePrefix::new(
            "customtransfer".parse().unwrap(),
            "channel-1".parse().unwrap(),
        );

        let trace_prefix_3 =
            TracePrefix::new("transferv2".parse().unwrap(), "channel-10".parse().unwrap());
        let trace_prefix_4 = TracePrefix::new(
            "transferv3".parse().unwrap(),
            "channel-101".parse().unwrap(),
        );

        prefixed_denom.trace_path.add_prefix(trace_prefix_3.clone());

        assert!(!prefixed_denom.trace_path.starts_with(&trace_prefix_1));
        assert!(!prefixed_denom.trace_path.starts_with(&trace_prefix_2));
        assert!(prefixed_denom.trace_path.starts_with(&trace_prefix_3));
        assert!(!prefixed_denom.trace_path.starts_with(&trace_prefix_4));

        assert_eq!(
            prefixed_denom.to_string(),
            "transferv2/channel-10/customtransfer/channel-1/alternativetransfer/channel-2/uatom"
        );

        prefixed_denom.trace_path.remove_prefix(&trace_prefix_4);

        assert!(!prefixed_denom.trace_path.is_empty());
        assert!(!prefixed_denom.trace_path.starts_with(&trace_prefix_1));
        assert!(!prefixed_denom.trace_path.starts_with(&trace_prefix_2));
        assert!(prefixed_denom.trace_path.starts_with(&trace_prefix_3));
        assert!(!prefixed_denom.trace_path.starts_with(&trace_prefix_4));
        assert_eq!(
            prefixed_denom.to_string(),
            "transferv2/channel-10/customtransfer/channel-1/alternativetransfer/channel-2/uatom"
        );

        prefixed_denom.trace_path.remove_prefix(&trace_prefix_3);

        assert!(!prefixed_denom.trace_path.is_empty());
        assert!(!prefixed_denom.trace_path.starts_with(&trace_prefix_1));
        assert!(prefixed_denom.trace_path.starts_with(&trace_prefix_2));
        assert!(!prefixed_denom.trace_path.starts_with(&trace_prefix_3));
        assert!(!prefixed_denom.trace_path.starts_with(&trace_prefix_4));
        assert_eq!(
            prefixed_denom.to_string(),
            "customtransfer/channel-1/alternativetransfer/channel-2/uatom"
        );

        prefixed_denom.trace_path.remove_prefix(&trace_prefix_2);

        assert!(!prefixed_denom.trace_path.is_empty());
        assert!(prefixed_denom.trace_path.starts_with(&trace_prefix_1));
        assert!(!prefixed_denom.trace_path.starts_with(&trace_prefix_2));
        assert!(!prefixed_denom.trace_path.starts_with(&trace_prefix_3));
        assert!(!prefixed_denom.trace_path.starts_with(&trace_prefix_4));
        assert_eq!(
            prefixed_denom.to_string(),
            "alternativetransfer/channel-2/uatom"
        );

        prefixed_denom.trace_path.remove_prefix(&trace_prefix_1);

        assert!(prefixed_denom.trace_path.is_empty());
        assert!(!prefixed_denom.trace_path.starts_with(&trace_prefix_1));
        assert!(!prefixed_denom.trace_path.starts_with(&trace_prefix_2));
        assert!(!prefixed_denom.trace_path.starts_with(&trace_prefix_3));
        assert!(!prefixed_denom.trace_path.starts_with(&trace_prefix_4));
        assert_eq!(prefixed_denom.to_string(), "uatom");
    }

    #[rstest]
    #[case("", TracePath::empty(), Some(""))]
    #[case("transfer", TracePath::empty(), Some("transfer"))]
    #[case("transfer/", TracePath::empty(), Some("transfer/"))]
    #[case("transfer/channel-1", TracePath::from(vec![TracePrefix::new("transfer".parse().unwrap(), ChannelId::new(1))]), None)]
    #[case("transfer/channel-1/", TracePath::from(vec![TracePrefix::new("transfer".parse().unwrap(), ChannelId::new(1))]), Some(""))]
    #[case("transfer/channel-1/uatom", TracePath::from(vec![TracePrefix::new("transfer".parse().unwrap(), ChannelId::new(1))]), Some("uatom"))]
    #[case("transfer/channel-1/uatom/", TracePath::from(vec![TracePrefix::new("transfer".parse().unwrap(), ChannelId::new(1))]), Some("uatom/"))]
    fn test_trace_path_cases(
        #[case] trace_path_s: &str,
        #[case] trace_path: TracePath,
        #[case] remaining: Option<&str>,
    ) {
        let (parsed_trace_path, parsed_remaining) = TracePath::trim(trace_path_s);

        assert_eq!(parsed_trace_path, trace_path);
        assert_eq!(parsed_remaining, remaining);
    }

    #[test]
    fn test_trace_path() -> Result<(), TokenTransferError> {
        assert!(TracePath::from_str("").is_ok(), "empty trace path");
        assert!(
            TracePath::from_str("transfer/uatom").is_err(),
            "invalid trace path: bad ChannelId"
        );
        assert!(
            TracePath::from_str("transfer//uatom").is_err(),
            "malformed trace path: missing ChannelId"
        );
        assert!(
            TracePath::from_str("transfer/channel-0/").is_err(),
            "malformed trace path: trailing delimiter"
        );

        let prefix_1 = TracePrefix::new("transfer".parse().unwrap(), "channel-1".parse().unwrap());
        let prefix_2 = TracePrefix::new("transfer".parse().unwrap(), "channel-0".parse().unwrap());
        let mut trace_path = TracePath(vec![prefix_1.clone()]);

        trace_path.add_prefix(prefix_2.clone());
        assert_eq!(
            TracePath::from_str("transfer/channel-0/transfer/channel-1")?,
            trace_path
        );
        assert_eq!(
            TracePath(vec![prefix_1.clone(), prefix_2.clone()]),
            trace_path
        );

        trace_path.remove_prefix(&prefix_2);
        assert_eq!(TracePath::from_str("transfer/channel-1")?, trace_path);
        assert_eq!(TracePath(vec![prefix_1.clone()]), trace_path);

        trace_path.remove_prefix(&prefix_1);
        assert!(trace_path.is_empty());

        Ok(())
    }
}
