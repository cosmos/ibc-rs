//! Defines coin types; the objects that are being transferred.

use core::fmt::{Display, Error as FmtError, Formatter};
use core::str::FromStr;

use ibc_proto::cosmos::base::v1beta1::Coin as ProtoCoin;

use super::amount::Amount;
use super::denom::{BaseDenom, PrefixedDenom};
use super::error::TokenTransferError;
use crate::prelude::*;

/// A `Coin` type with fully qualified `PrefixedDenom`.
pub type PrefixedCoin = Coin<PrefixedDenom>;

/// A `Coin` type with an unprefixed denomination.
pub type BaseCoin = Coin<BaseDenom>;

pub type RawCoin = Coin<String>;

/// Coin defines a token with a denomination and an amount.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode,)
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Coin<D> {
    /// Denomination
    pub denom: D,
    /// Amount
    pub amount: Amount,
}

impl<D> Coin<D> {
    pub fn new<T, DT>(amount: T, denom: DT) -> Self
    where
        T: Into<Amount>,
        DT: Into<D>,
    {
        Self {
            denom: denom.into(),
            amount: amount.into(),
        }
    }
}

impl<D: FromStr> Coin<D>
where
    D::Err: Into<TokenTransferError>,
{
    pub fn from_string_list(coin_str: &str) -> Result<Vec<Self>, TokenTransferError> {
        coin_str.split(',').map(FromStr::from_str).collect()
    }
}

impl<D: FromStr> FromStr for Coin<D>
where
    D::Err: Into<TokenTransferError>,
{
    type Err = TokenTransferError;

    #[allow(clippy::assign_op_pattern)]
    fn from_str(coin_str: &str) -> Result<Self, TokenTransferError> {
        // Denominations can be 3 ~ 128 characters long and support letters, followed by either
        // a letter, a number or a separator ('/', ':', '.', '_' or '-').
        // Loosely copy the regex from here:
        // https://github.com/cosmos/cosmos-sdk/blob/v0.45.5/types/coin.go#L760-L762
        // old regex: ^(?<amount>[0-9]+)(?<denom>[a-zA-Z0-9/:\\._-]+)$

        let (amount, denom) = coin_str
            .chars()
            .position(|x| !x.is_numeric())
            .map(|index| coin_str.split_at(index))
            .filter(|(amount, denom)| !amount.is_empty() && !denom.is_empty())
            .filter(|(_, denom)| {
                !denom.contains(|x| {
                    !matches!(x, 'a'..='z' | 'A'..='Z' | '0'..='9' | '/' | ':' | '.' | '_' | '-')
                })
            })
            .ok_or_else(|| TokenTransferError::InvalidCoin {
                coin: coin_str.to_string(),
            })?;

        Ok(Coin {
            amount: amount.parse()?,
            denom: denom.parse().map_err(Into::into)?,
        })
    }
}

impl<D: FromStr> TryFrom<ProtoCoin> for Coin<D>
where
    D::Err: Into<TokenTransferError>,
{
    type Error = TokenTransferError;

    fn try_from(proto: ProtoCoin) -> Result<Coin<D>, Self::Error> {
        let denom = D::from_str(&proto.denom).map_err(Into::into)?;
        let amount = Amount::from_str(&proto.amount)?;
        Ok(Self { denom, amount })
    }
}

impl<D: ToString> From<Coin<D>> for ProtoCoin {
    fn from(coin: Coin<D>) -> ProtoCoin {
        ProtoCoin {
            denom: coin.denom.to_string(),
            amount: coin.amount.to_string(),
        }
    }
}

impl From<BaseCoin> for PrefixedCoin {
    fn from(coin: BaseCoin) -> PrefixedCoin {
        PrefixedCoin {
            denom: coin.denom.into(),
            amount: coin.amount,
        }
    }
}

impl<D: Display> Display for Coin<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}{}", self.amount, self.denom)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("123stake", RawCoin::new(123, "stake"))]
    #[case("1a1", RawCoin::new(1, "a1"))]
    #[case("0x1/:.\\_-", RawCoin::new(0, "x1/:.\\_-"))]
    fn test_parse_raw_coin(#[case] parsed: RawCoin, #[case] expected: RawCoin) {
        assert_eq!(parsed, expected);
    }

    #[rstest]
    #[case("0x!")]
    #[should_panic]
    fn test_failed_parse_raw_coin(#[case] raw: &str) {
        RawCoin::from_str(raw).expect("parsing failure");
    }

    #[rstest]
    #[case("123stake,1a1,999den0m", &[RawCoin::new(123, "stake"), RawCoin::new(1, "a1"), RawCoin::new(999, "den0m")])]
    fn test_parse_raw_coin_list(
        #[case] coins_str: &str,
        #[case] coins: &[RawCoin],
    ) -> Result<(), TokenTransferError> {
        assert_eq!(RawCoin::from_string_list(coins_str)?, coins);
        Ok(())
    }
}
