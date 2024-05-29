use core::str::FromStr;

use alloc::string::{String, ToString};
use ibc_proto::ibc::applications::interchain_accounts::v1::Metadata as RawMetadata;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use tendermint_proto::Protobuf;

use super::context::InterchainAccountValidationContext;
use super::error::InterchainAccountError;
use super::VERSION;
use crate::core::ics04_channel::Version;
use crate::core::ics24_host::identifier::ConnectionId;
use crate::Signer;

/// Defines a set of protocol specific data encoded into the ICS27 channel version bytestring
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata {
    /// Defines the ICS27 protocol version
    pub version: Version,
    /// Defines the connection identifier associated with the controller chain
    pub conn_id_on_a: ConnectionId,
    /// Defines the connection identifier associated with the host chain
    pub conn_id_on_b: ConnectionId,
    /// Defines the interchain account address to be fulfilled upon the OnChanOpenTry handshake step
    /// NOTE: the address field is empty on the `OnChanOpenInit` handshake step
    pub address: Signer,
    /// Defines the supported codec format
    pub encoding: SupportedEncoding,
    /// Defines the type of transactions the interchain account can execute
    pub tx_type: SupportedTxType,
}

impl Metadata {
    /// Constructs a new Metadata instance
    pub fn new(
        version: Version,
        conn_id_on_a: ConnectionId,
        conn_id_on_b: ConnectionId,
        address: Signer,
        encoding: SupportedEncoding,
        tx_type: SupportedTxType,
    ) -> Self {
        Metadata {
            version,
            conn_id_on_a,
            conn_id_on_b,
            address,
            encoding,
            tx_type,
        }
    }

    /// Constructs a new Metadata instance with default values
    pub fn new_default(conn_id_on_a: ConnectionId, conn_id_on_b: ConnectionId) -> Self {
        Self::new(
            Version::from(VERSION.to_string()),
            conn_id_on_a,
            conn_id_on_b,
            Signer::new_empty(),
            SupportedEncoding::Proto3,
            SupportedTxType::SDKMultiMsg,
        )
    }

    /// Validate the metadata using the provided validation context and connection hops
    pub fn validate(
        &self,
        ctx: &impl InterchainAccountValidationContext,
        connection_hops: &[ConnectionId],
    ) -> Result<(), InterchainAccountError> {
        let expected_metadata_version = Version::from(VERSION.to_string());

        if self.version != expected_metadata_version {
            return Err(
                InterchainAccountError::mismatch("channel version mismatch.")
                    .given(&self.version)
                    .expected(&expected_metadata_version),
            );
        }

        if self.encoding != SupportedEncoding::Proto3 {
            return Err(InterchainAccountError::not_supported("encoding type"));
        }

        if self.tx_type != SupportedTxType::SDKMultiMsg {
            return Err(InterchainAccountError::not_supported("tx type"));
        }

        if self.conn_id_on_a != connection_hops[0] {
            return Err(InterchainAccountError::mismatch("connection id mismatch.")
                .given(&self.conn_id_on_a)
                .expected(&connection_hops[0]));
        }

        let conn_end_on_a = ctx.connection_end(&connection_hops[0])?;

        let conn_id_on_b = conn_end_on_a.counterparty().clone().connection_id.ok_or(
            InterchainAccountError::not_found("connection id on counterparty"),
        )?;

        if self.conn_id_on_b != conn_id_on_b {
            return Err(InterchainAccountError::mismatch("connection id mismatch.")
                .given(&self.conn_id_on_b)
                .expected(&conn_id_on_b));
        }

        ctx.validate_message_signer(&self.address)?;

        Ok(())
    }

    // Compares a metadata to a previous version string set in a channel struct.
    // It ensures all fields are equal except the Address string
    pub fn verify_prev_metadata_matches(
        &self,
        previous_version: &Version,
    ) -> Result<(), InterchainAccountError> {
        let previous_metadata = serde_json::from_str::<Metadata>(previous_version.as_str())
            .map_err(InterchainAccountError::source)?;

        if self.version != previous_metadata.version {
            return Err(InterchainAccountError::mismatch("channel version")
                .given(&previous_metadata.version)
                .expected(&self.version));
        }

        if self.encoding != previous_metadata.encoding {
            return Err(InterchainAccountError::mismatch("encoding type")
                .given(&previous_metadata.encoding)
                .expected(&self.encoding));
        }

        if self.tx_type != previous_metadata.tx_type {
            return Err(InterchainAccountError::mismatch("tx type")
                .given(&previous_metadata.tx_type)
                .expected(&self.tx_type));
        }

        if self.conn_id_on_a != previous_metadata.conn_id_on_a {
            return Err(
                InterchainAccountError::mismatch("connection id on the controller chain")
                    .given(&previous_metadata.conn_id_on_a)
                    .expected(&self.conn_id_on_a),
            );
        }

        if self.conn_id_on_b != previous_metadata.conn_id_on_b {
            return Err(
                InterchainAccountError::mismatch("connection id on the host chain")
                    .given(&previous_metadata.conn_id_on_b)
                    .expected(&self.conn_id_on_b),
            );
        }

        Ok(())
    }
}

impl Protobuf<RawMetadata> for Metadata {}

impl From<Metadata> for RawMetadata {
    fn from(domain: Metadata) -> Self {
        RawMetadata {
            version: domain.version.to_string(),
            controller_connection_id: domain.conn_id_on_a.to_string(),
            host_connection_id: domain.conn_id_on_b.to_string(),
            address: domain.address.to_string(),
            encoding: domain.encoding.to_string(),
            tx_type: domain.tx_type.to_string(),
        }
    }
}

impl TryFrom<RawMetadata> for Metadata {
    type Error = InterchainAccountError;

    fn try_from(raw: RawMetadata) -> Result<Self, Self::Error> {
        Ok(Metadata {
            version: raw.version.parse().unwrap(),
            conn_id_on_a: raw
                .controller_connection_id
                .parse()
                .map_err(InterchainAccountError::source)?,
            conn_id_on_b: raw
                .host_connection_id
                .parse()
                .map_err(InterchainAccountError::source)?,
            address: Signer::new(raw.address),
            encoding: raw.encoding.parse()?,
            tx_type: raw.tx_type.parse()?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupportedEncoding {
    Proto3,
}

impl FromStr for SupportedEncoding {
    type Err = InterchainAccountError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "proto3" => Ok(SupportedEncoding::Proto3),
            _ => Err(InterchainAccountError::not_supported(
                "supported encoding type",
            )),
        }
    }
}

impl ToString for SupportedEncoding {
    fn to_string(&self) -> String {
        match self {
            SupportedEncoding::Proto3 => "proto3".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupportedTxType {
    SDKMultiMsg,
}

impl FromStr for SupportedTxType {
    type Err = InterchainAccountError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sdk_multi_msg" => Ok(SupportedTxType::SDKMultiMsg),
            _ => Err(InterchainAccountError::not_supported("supported tx type")),
        }
    }
}

impl ToString for SupportedTxType {
    fn to_string(&self) -> String {
        match self {
            SupportedTxType::SDKMultiMsg => "sdk_multi_msg".to_string(),
        }
    }
}
