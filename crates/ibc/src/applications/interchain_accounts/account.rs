use alloc::string::{String, ToString};
use alloc::vec;
use cosmrs::AccountId;
use prost::Message;

use ibc_proto::cosmos::auth::v1beta1::BaseAccount as RawBaseAccount;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::applications::interchain_accounts::v1::InterchainAccount as RawInterchainAccount;
use ibc_proto::protobuf::Protobuf;
use sha2::{Digest, Sha256};

use super::error::InterchainAccountError;
use super::MODULE_ID_STR;
use crate::core::ics24_host::identifier::PortId;
use crate::Signer;

/// Defines an interchain account type with a generic base account.
///
/// TODO: to put a note that we currently only support Cosmos-SDK driven chains.
#[derive(Clone, Debug)]
pub struct InterchainAccount<A: BaseAccount> {
    /// The base account.
    base_account: A,
    /// The account owner.
    owner: PortId,
}

impl<A: BaseAccount> InterchainAccount<A> {
    /// Constructs a new interchain account instance.
    pub fn new(base_account: A, owner: PortId) -> Self {
        Self {
            base_account,
            owner,
        }
    }

    /// Constructs a new interchain account with a Cosmos-SDK base account.
    pub fn new_with_sdk_base_account(
        address: AccountId,
        owner: PortId,
    ) -> InterchainAccount<SdkBaseAccount> {
        let acc = SdkBaseAccount {
            address,
            pub_key: Any {
                type_url: String::new(),
                value: vec![],
            },
            account_number: 0,
            sequence: 0,
        };
        InterchainAccount::new(acc, owner)
    }

    pub fn address(&self) -> Signer {
        self.base_account.address()
    }
}

impl BaseAccount for SdkBaseAccount {
    fn address(&self) -> Signer {
        Signer::from(self.address.to_string())
    }
}

impl Protobuf<RawInterchainAccount> for InterchainAccount<SdkBaseAccount> {}

impl TryFrom<RawInterchainAccount> for InterchainAccount<SdkBaseAccount> {
    type Error = InterchainAccountError;

    fn try_from(raw: RawInterchainAccount) -> Result<Self, Self::Error> {
        Ok(InterchainAccount {
            base_account: match raw.base_account {
                Some(base_account) => SdkBaseAccount::try_from(base_account)?,
                None => return Err(InterchainAccountError::not_found("base account")),
            },
            owner: raw.account_owner.parse().unwrap(),
        })
    }
}

impl From<InterchainAccount<SdkBaseAccount>> for RawInterchainAccount {
    fn from(domain: InterchainAccount<SdkBaseAccount>) -> Self {
        RawInterchainAccount {
            base_account: Some(domain.base_account.into()),
            account_owner: domain.owner.to_string(),
        }
    }
}

/// Defines the base account for Cosmos-SDK driven chains.
#[derive(Clone, Debug)]
pub struct SdkBaseAccount {
    /// The address of the account.
    pub address: AccountId,
    /// The public key of the account.
    pub pub_key: Any,
    /// The account number.
    pub account_number: u64,
    /// The sequence number.
    pub sequence: u64,
}

impl Protobuf<RawBaseAccount> for SdkBaseAccount {}

impl TryFrom<RawBaseAccount> for SdkBaseAccount {
    type Error = InterchainAccountError;

    fn try_from(raw: RawBaseAccount) -> Result<Self, Self::Error> {
        // TODO: should we check anything here? regarding number and sequence?
        Ok(SdkBaseAccount {
            address: raw
                .address
                .parse()
                .map_err(InterchainAccountError::source)?,
            pub_key: match raw.pub_key {
                Some(pub_key) => pub_key,
                None => return Err(InterchainAccountError::not_found("missing base account")),
            },
            account_number: raw.account_number,
            sequence: raw.sequence,
        })
    }
}

impl From<SdkBaseAccount> for RawBaseAccount {
    fn from(domain: SdkBaseAccount) -> Self {
        RawBaseAccount {
            address: domain.address.to_string(),
            pub_key: Some(domain.pub_key),
            account_number: domain.account_number,
            sequence: domain.sequence,
        }
    }
}

const TYPE_URL: &str = "/cosmos.auth.v1beta1.BaseAccount";

impl From<SdkBaseAccount> for Any {
    fn from(account: SdkBaseAccount) -> Self {
        let account = RawBaseAccount::from(account);
        Any {
            type_url: TYPE_URL.to_string(),
            value: account.encode_to_vec(),
        }
    }
}

/// Enforces minimum definition requirement for a base account.
pub trait BaseAccount {
    fn address(&self) -> Signer;
}

pub fn get_sdk_controller_account() -> Result<AccountId, InterchainAccountError> {
    let mut hasher = Sha256::new();

    hasher.update(MODULE_ID_STR.as_bytes());

    let mut hash = hasher.finalize().to_vec();

    hash.truncate(20);

    let controller_account =
        AccountId::new(MODULE_ID_STR, &hash).map_err(InterchainAccountError::source)?;

    Ok(controller_account)
}
