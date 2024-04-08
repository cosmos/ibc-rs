//! Defines the domain type for tendermint headers

use core::fmt::{Display, Error as FmtError, Formatter};
use core::str::FromStr;

use ibc_core_client_types::error::ClientError;
use ibc_core_client_types::Height;
use ibc_core_host_types::identifiers::ChainId;
use ibc_primitives::prelude::*;
use ibc_primitives::Timestamp;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::lightclients::tendermint::v1::Header as RawHeader;
use ibc_proto::Protobuf;
use pretty::{PrettySignedHeader, PrettyValidatorSet};
use tendermint::block::signed_header::SignedHeader;
use tendermint::chain::Id as TmChainId;
use tendermint::crypto::Sha256;
use tendermint::merkle::MerkleHash;
use tendermint::validator::Set as ValidatorSet;
use tendermint::{Hash, Time};
use tendermint_light_client_verifier::types::{TrustedBlockState, UntrustedBlockState};

use crate::error::Error;

pub const TENDERMINT_HEADER_TYPE_URL: &str = "/ibc.lightclients.tendermint.v1.Header";

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Tendermint consensus header
#[derive(Clone, PartialEq, Eq)]
pub struct Header {
    pub signed_header: SignedHeader, // contains the commitment root
    pub validator_set: ValidatorSet, // the validator set that signed Header
    pub trusted_height: Height, // the height of a trusted header seen by client less than or equal to Header
    pub trusted_next_validator_set: ValidatorSet, // the last trusted validator set at trusted height
}

impl core::fmt::Debug for Header {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, " Header {{...}}")
    }
}

impl Display for Header {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "Header {{ signed_header: {}, validator_set: {}, trusted_height: {}, trusted_validator_set: {} }}", PrettySignedHeader(&self.signed_header), PrettyValidatorSet(&self.validator_set), self.trusted_height, PrettyValidatorSet(&self.trusted_next_validator_set))
    }
}

impl Header {
    pub fn timestamp(&self) -> Timestamp {
        self.signed_header.header.time.into()
    }

    pub fn height(&self) -> Height {
        Height::new(
            ChainId::from_str(self.signed_header.header.chain_id.as_str())
                .expect("chain id")
                .revision_number(),
            u64::from(self.signed_header.header.height),
        )
        .expect("malformed tendermint header domain type has an illegal height of 0")
    }

    pub fn as_untrusted_block_state(&self) -> UntrustedBlockState<'_> {
        UntrustedBlockState {
            signed_header: &self.signed_header,
            validators: &self.validator_set,
            next_validators: None,
        }
    }

    pub fn as_trusted_block_state<'a>(
        &'a self,
        chain_id: &'a TmChainId,
        header_time: Time,
        next_validators_hash: Hash,
    ) -> Result<TrustedBlockState<'a>, Error> {
        Ok(TrustedBlockState {
            chain_id,
            header_time,
            height: self
                .trusted_height
                .revision_height()
                .try_into()
                .map_err(|_| Error::InvalidHeaderHeight {
                    height: self.trusted_height.revision_height(),
                })?,
            next_validators: &self.trusted_next_validator_set,
            next_validators_hash,
        })
    }

    pub fn verify_chain_id_version_matches_height(&self, chain_id: &ChainId) -> Result<(), Error> {
        if self.height().revision_number() != chain_id.revision_number() {
            return Err(Error::MismatchHeaderChainId {
                given: self.signed_header.header.chain_id.to_string(),
                expected: chain_id.to_string(),
            });
        }
        Ok(())
    }

    /// `header.trusted_next_validator_set` was given to us by the relayer.
    /// Thus, we need to ensure that the relayer gave us the right set, i.e. by
    /// ensuring that it matches the hash we have stored on chain.
    pub fn check_trusted_next_validator_set<H: MerkleHash + Sha256 + Default>(
        &self,
        trusted_next_validator_hash: &Hash,
    ) -> Result<(), ClientError> {
        if &self.trusted_next_validator_set.hash_with::<H>() == trusted_next_validator_hash {
            Ok(())
        } else {
            Err(ClientError::HeaderVerificationFailure {
                reason:
                    "header trusted next validator set hash does not match hash stored on chain"
                        .to_string(),
            })
        }
    }

    /// Checks if the fields of a given header are consistent with the trusted fields of this header.
    pub fn validate_basic<H: MerkleHash + Sha256 + Default>(&self) -> Result<(), Error> {
        if self.height().revision_number() != self.trusted_height.revision_number() {
            return Err(Error::MismatchHeightRevisions {
                trusted_revision: self.trusted_height.revision_number(),
                header_revision: self.height().revision_number(),
            });
        }

        // We need to ensure that the trusted height (representing the
        // height of the header already on chain for which this client update is
        // based on) must be smaller than height of the new header that we're
        // installing.
        if self.trusted_height >= self.height() {
            return Err(Error::InvalidHeaderHeight {
                height: self.height().revision_height(),
            });
        }

        let validators_hash = self.validator_set.hash_with::<H>();

        if validators_hash != self.signed_header.header.validators_hash {
            return Err(Error::MismatchValidatorsHashes {
                signed_header_validators_hash: self.signed_header.header.validators_hash,
                validators_hash,
            });
        }

        Ok(())
    }
}

impl Protobuf<RawHeader> for Header {}

impl TryFrom<RawHeader> for Header {
    type Error = Error;

    fn try_from(raw: RawHeader) -> Result<Self, Self::Error> {
        let header = Self {
            signed_header: raw
                .signed_header
                .ok_or(Error::MissingSignedHeader)?
                .try_into()
                .map_err(|e| Error::InvalidHeader {
                    reason: "signed header conversion".to_string(),
                    error: e,
                })?,
            validator_set: raw
                .validator_set
                .ok_or(Error::MissingValidatorSet)?
                .try_into()
                .map_err(Error::InvalidRawHeader)?,
            trusted_height: raw
                .trusted_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(Error::MissingTrustedHeight)?,
            trusted_next_validator_set: raw
                .trusted_validators
                .ok_or(Error::MissingTrustedNextValidatorSet)?
                .try_into()
                .map_err(Error::InvalidRawHeader)?,
        };

        Ok(header)
    }
}

impl Protobuf<Any> for Header {}

impl TryFrom<Any> for Header {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        fn decode_header(value: &[u8]) -> Result<Header, ClientError> {
            let header = Protobuf::<RawHeader>::decode(value).map_err(|e| ClientError::Other {
                description: e.to_string(),
            })?;
            Ok(header)
        }
        match raw.type_url.as_str() {
            TENDERMINT_HEADER_TYPE_URL => decode_header(&raw.value),
            _ => Err(ClientError::UnknownHeaderType {
                header_type: raw.type_url,
            }),
        }
    }
}

impl From<Header> for Any {
    fn from(header: Header) -> Self {
        Any {
            type_url: TENDERMINT_HEADER_TYPE_URL.to_string(),
            value: Protobuf::<RawHeader>::encode_vec(header),
        }
    }
}

impl From<Header> for RawHeader {
    fn from(value: Header) -> Self {
        RawHeader {
            signed_header: Some(value.signed_header.into()),
            validator_set: Some(value.validator_set.into()),
            trusted_height: Some(value.trusted_height.into()),
            trusted_validators: Some(value.trusted_next_validator_set.into()),
        }
    }
}

mod pretty {
    use ibc_primitives::utils::PrettySlice;

    pub use super::*;

    pub struct PrettySignedHeader<'a>(pub &'a SignedHeader);

    impl Display for PrettySignedHeader<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
            write!(
            f,
            "SignedHeader {{ header: {{ chain_id: {}, height: {} }}, commit: {{ height: {} }} }}",
            self.0.header.chain_id, self.0.header.height, self.0.commit.height
        )
        }
    }

    pub struct PrettyValidatorSet<'a>(pub &'a ValidatorSet);

    impl Display for PrettyValidatorSet<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
            let validator_addresses: Vec<_> = self
                .0
                .validators()
                .iter()
                .map(|validator| validator.address)
                .collect();
            if let Some(proposer) = self.0.proposer() {
                match &proposer.name {
                Some(name) => write!(f, "PrettyValidatorSet {{ validators: {}, proposer: {}, total_voting_power: {} }}", PrettySlice(&validator_addresses), name, self.0.total_voting_power()),
                None =>  write!(f, "PrettyValidatorSet {{ validators: {}, proposer: None, total_voting_power: {} }}", PrettySlice(&validator_addresses), self.0.total_voting_power()),
            }
            } else {
                write!(
                f,
                "PrettyValidatorSet {{ validators: {}, proposer: None, total_voting_power: {} }}",
                PrettySlice(&validator_addresses),
                self.0.total_voting_power()
            )
            }
        }
    }
}
