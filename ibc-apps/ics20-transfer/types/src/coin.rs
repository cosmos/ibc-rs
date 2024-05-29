//! Defines coin types; the objects that are being transferred.
use core::fmt::{Display, Error as FmtError, Formatter};
use core::str::FromStr;

use ibc_core::primitives::prelude::*;
use ibc_proto::cosmos::base::v1beta1::Coin as ProtoCoin;

use super::amount::Amount;
use super::denom::{BaseDenom, PrefixedDenom};
use super::error::TokenTransferError;

/// A `Coin` type with fully qualified `PrefixedDenom`.
pub type PrefixedCoin = Coin<PrefixedDenom>;

/// A `Coin` type with an unprefixed denomination.
pub type BaseCoin = Coin<BaseDenom>;

pub type RawCoin = Coin<String>;

/// Allowed characters in string representation of a denomination.
const VALID_DENOM_CHARACTERS: &str = "/:._-";

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
        // https://github.com/cosmos/cosmos-sdk/blob/v0.47.5/types/coin.go#L838-L840
        //
        // equivalent regex code in rust:
        // let re = Regex::new(r"^(?<amount>[0-9]+)(?<denom>[a-zA-Z0-9/:._-]+)$")?;
        // let cap = re.captures("123stake")?;
        // let (amount, denom) = (cap.name("amount")?.as_str(), cap.name("denom")?.as_str());

        let (amount, denom) = coin_str
            .chars()
            .position(|x| !x.is_numeric())
            .map(|index| coin_str.split_at(index))
            .filter(|(amount, _)| !amount.is_empty())
            .filter(|(_, denom)| {
                denom
                    .chars()
                    .all(|x| x.is_alphanumeric() || VALID_DENOM_CHARACTERS.contains(x))
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
    use primitive_types::U256;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::nat("123stake", 123, "stake")]
    #[case::zero("0stake", 0, "stake")]
    #[case::u256_max(
        "115792089237316195423570985008687907853269984665640564039457584007913129639935stake",
        U256::MAX,
        "stake"
    )]
    #[case::digit_in_denom("1a1", 1, "a1")]
    #[case::chars_in_denom("0x1/:._-", 0, "x1/:._-")]
    #[case::ibc_denom("1234ibc/a0B1C", 1234, "ibc/a0B1C")]
    fn test_parse_raw_coin(
        #[case] parsed: RawCoin,
        #[case] amount: impl Into<Amount>,
        #[case] denom: &str,
    ) {
        assert_eq!(
            parsed,
            RawCoin {
                denom: denom.into(),
                amount: amount.into()
            }
        );
    }

    #[rstest]
    #[case::pos("+123stake")]
    #[case::pos_zero("+0stake")]
    #[case::neg("-123stake")]
    #[case::neg_zero("-0stake")]
    #[case::u256_max_plus_1(
        "115792089237316195423570985008687907853269984665640564039457584007913129639936stake"
    )]
    #[case::invalid_char_in_denom("0x!")]
    #[case::blackslash_in_denom("0x1/:.\\_-")]
    #[should_panic]
    fn test_failed_parse_raw_coin(#[case] _raw: RawCoin) {}

    #[rstest]
    #[case::nomal("123stake,1a1,999den0m", &[(123, "stake"), (1, "a1"), (999, "den0m")])]
    #[case::tricky("123stake,1a1-999den0m", &[(123, "stake"), (1, "a1-999den0m")])]
    #[case::colon_delimiter("123stake:1a1:999den0m", &[(123, "stake:1a1:999den0m")])]
    #[case::dash_delimiter("123stake-1a1-999den0m", &[(123, "stake-1a1-999den0m")])]
    #[case::slash_delimiter("123stake/1a1/999den0m", &[(123, "stake/1a1/999den0m")])]
    fn test_parse_raw_coin_list(
        #[case] coins_str: &str,
        #[case] coins: &[(u64, &str)],
    ) -> Result<(), TokenTransferError> {
        assert_eq!(
            RawCoin::from_string_list(coins_str)?,
            coins
                .iter()
                .map(|&(amount, denom)| RawCoin {
                    denom: denom.to_string(),
                    amount: amount.into(),
                })
                .collect::<Vec<_>>()
        );
        Ok(())
    }

    #[rstest]
    #[case::semicolon_delimiter("123stake;1a1;999den0m")]
    #[case::mixed_delimiter("123stake,1a1;999den0m")]
    #[should_panic(expected = "parsing failure in test")]
    fn test_failed_parse_raw_coin_list(#[case] coins_str: &str) {
        RawCoin::from_string_list(coins_str).expect("parsing failure in test");
    }
}
