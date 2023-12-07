#![feature(prelude_import)]
//! ICS-24: Host defines the minimal set of interfaces that a state machine
//! hosting an IBC-enabled chain must implement.
#![no_std]
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::disallowed_methods, clippy::disallowed_types)]
#![deny(
    warnings,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]
#[prelude_import]
use core::prelude::rust_2021::*;
#[macro_use]
extern crate core;
extern crate compiler_builtins as _;
#[cfg(feature = "std")]
extern crate std;
pub mod error {
    use displaydoc::Display;
    use ibc_primitives::prelude::*;
    pub enum IdentifierError {
        /// identifier `{id}` has invalid length; must be between `{min}` and `{max}` characters
        InvalidLength { id: String, min: u64, max: u64 },
        /// identifier `{id}` must only contain alphanumeric characters or `.`, `_`, `+`, `-`, `#`, - `[`, `]`, `<`, `>`
        InvalidCharacter { id: String },
        /// identifier prefix `{prefix}` is invalid
        InvalidPrefix { prefix: String },
        /// chain identifier is not formatted with revision number
        UnformattedRevisionNumber { chain_id: String },
        /// revision number overflowed
        RevisionNumberOverflow,
        /// String `{value}` cannot be converted to packet sequence, error: `{reason}`
        InvalidStringAsSequence { value: String, reason: String },
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for IdentifierError {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                IdentifierError::InvalidLength {
                    id: __self_0,
                    min: __self_1,
                    max: __self_2,
                } => {
                    ::core::fmt::Formatter::debug_struct_field3_finish(
                        f,
                        "InvalidLength",
                        "id",
                        __self_0,
                        "min",
                        __self_1,
                        "max",
                        &__self_2,
                    )
                }
                IdentifierError::InvalidCharacter { id: __self_0 } => {
                    ::core::fmt::Formatter::debug_struct_field1_finish(
                        f,
                        "InvalidCharacter",
                        "id",
                        &__self_0,
                    )
                }
                IdentifierError::InvalidPrefix { prefix: __self_0 } => {
                    ::core::fmt::Formatter::debug_struct_field1_finish(
                        f,
                        "InvalidPrefix",
                        "prefix",
                        &__self_0,
                    )
                }
                IdentifierError::UnformattedRevisionNumber { chain_id: __self_0 } => {
                    ::core::fmt::Formatter::debug_struct_field1_finish(
                        f,
                        "UnformattedRevisionNumber",
                        "chain_id",
                        &__self_0,
                    )
                }
                IdentifierError::RevisionNumberOverflow => {
                    ::core::fmt::Formatter::write_str(f, "RevisionNumberOverflow")
                }
                IdentifierError::InvalidStringAsSequence {
                    value: __self_0,
                    reason: __self_1,
                } => {
                    ::core::fmt::Formatter::debug_struct_field2_finish(
                        f,
                        "InvalidStringAsSequence",
                        "value",
                        __self_0,
                        "reason",
                        &__self_1,
                    )
                }
            }
        }
    }
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _DERIVE_Display_FOR_IdentifierError: () = {
        trait DisplayToDisplayDoc {
            fn __displaydoc_display(&self) -> Self;
        }
        impl<T: core::fmt::Display> DisplayToDisplayDoc for &T {
            fn __displaydoc_display(&self) -> Self {
                self
            }
        }
        extern crate std;
        trait PathToDisplayDoc {
            fn __displaydoc_display(&self) -> std::path::Display<'_>;
        }
        impl PathToDisplayDoc for std::path::Path {
            fn __displaydoc_display(&self) -> std::path::Display<'_> {
                self.display()
            }
        }
        impl PathToDisplayDoc for std::path::PathBuf {
            fn __displaydoc_display(&self) -> std::path::Display<'_> {
                self.display()
            }
        }
        impl core::fmt::Display for IdentifierError {
            fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                #[allow(unused_variables)]
                match self {
                    Self::InvalidLength { id, min, max } => {
                        formatter
                            .write_fmt(
                                format_args!(
                                    "identifier `{0}` has invalid length; must be between `{1}` and `{2}` characters",
                                    id.__displaydoc_display(),
                                    min.__displaydoc_display(),
                                    max.__displaydoc_display(),
                                ),
                            )
                    }
                    Self::InvalidCharacter { id } => {
                        formatter
                            .write_fmt(
                                format_args!(
                                    "identifier `{0}` must only contain alphanumeric characters or `.`, `_`, `+`, `-`, `#`, - `[`, `]`, `<`, `>`",
                                    id.__displaydoc_display(),
                                ),
                            )
                    }
                    Self::InvalidPrefix { prefix } => {
                        formatter
                            .write_fmt(
                                format_args!(
                                    "identifier prefix `{0}` is invalid",
                                    prefix.__displaydoc_display(),
                                ),
                            )
                    }
                    Self::UnformattedRevisionNumber { chain_id } => {
                        formatter
                            .write_fmt(
                                format_args!(
                                    "chain identifier is not formatted with revision number",
                                ),
                            )
                    }
                    Self::RevisionNumberOverflow => {
                        formatter.write_fmt(format_args!("revision number overflowed"))
                    }
                    Self::InvalidStringAsSequence { value, reason } => {
                        formatter
                            .write_fmt(
                                format_args!(
                                    "String `{0}` cannot be converted to packet sequence, error: `{1}`",
                                    value.__displaydoc_display(),
                                    reason.__displaydoc_display(),
                                ),
                            )
                    }
                }
            }
        }
    };
    #[cfg(feature = "std")]
    impl std::error::Error for IdentifierError {}
}
pub mod identifiers {
    //! Defines identifier types
    mod chain_id {
        use core::fmt::{Debug, Display, Error as FmtError, Formatter};
        use core::str::FromStr;
        use ibc_primitives::prelude::*;
        use crate::error::IdentifierError;
        use crate::validate::{
            validate_identifier_chars, validate_identifier_length, validate_prefix_length,
        };
        /// Defines the domain type for chain identifiers.
        ///
        /// A valid `ChainId` follows the format {chain name}-{revision number} where
        /// the revision number indicates how many times the chain has been upgraded.
        /// Creating `ChainId`s not in this format will result in an error.
        ///
        /// It should be noted this format is not standardized yet, though it is widely
        /// accepted and compatible with Cosmos SDK driven chains.
        pub struct ChainId {
            id: String,
            revision_number: u64,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for ChainId {
            #[inline]
            fn clone(&self) -> ChainId {
                ChainId {
                    id: ::core::clone::Clone::clone(&self.id),
                    revision_number: ::core::clone::Clone::clone(&self.revision_number),
                }
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for ChainId {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "ChainId",
                    "id",
                    &self.id,
                    "revision_number",
                    &&self.revision_number,
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for ChainId {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for ChainId {
            #[inline]
            fn eq(&self, other: &ChainId) -> bool {
                self.id == other.id && self.revision_number == other.revision_number
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for ChainId {}
        #[automatically_derived]
        impl ::core::cmp::Eq for ChainId {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<String>;
                let _: ::core::cmp::AssertParamIsEq<u64>;
            }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialOrd for ChainId {
            #[inline]
            fn partial_cmp(
                &self,
                other: &ChainId,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                match ::core::cmp::PartialOrd::partial_cmp(&self.id, &other.id) {
                    ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                        ::core::cmp::PartialOrd::partial_cmp(
                            &self.revision_number,
                            &other.revision_number,
                        )
                    }
                    cmp => cmp,
                }
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Ord for ChainId {
            #[inline]
            fn cmp(&self, other: &ChainId) -> ::core::cmp::Ordering {
                match ::core::cmp::Ord::cmp(&self.id, &other.id) {
                    ::core::cmp::Ordering::Equal => {
                        ::core::cmp::Ord::cmp(
                            &self.revision_number,
                            &other.revision_number,
                        )
                    }
                    cmp => cmp,
                }
            }
        }
        #[automatically_derived]
        impl ::core::hash::Hash for ChainId {
            #[inline]
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                ::core::hash::Hash::hash(&self.id, state);
                ::core::hash::Hash::hash(&self.revision_number, state)
            }
        }
        impl ChainId {
            /// Creates a new `ChainId` with the given chain identifier.
            ///
            /// It checks the identifier for valid characters according to `ICS-24`
            /// specification and returns a `ChainId` successfully.
            /// Stricter checks beyond `ICS-24` rests with the users,
            /// based on their requirements.
            ///
            /// If the chain identifier is in the {chain name}-{revision number} format,
            /// the revision number is parsed. Otherwise, revision number is set to 0.
            ///
            /// ```
            /// use ibc_core_host_types::identifiers::ChainId;
            ///
            /// let chain_id = "chainA";
            /// let id = ChainId::new(chain_id).unwrap();
            /// assert_eq!(id.revision_number(), 0);
            /// assert_eq!(id.as_str(), chain_id);
            ///
            /// let chain_id = "chainA-12";
            /// let id = ChainId::new(chain_id).unwrap();
            /// assert_eq!(id.revision_number(), 12);
            /// assert_eq!(id.as_str(), chain_id);
            /// ```
            pub fn new(chain_id: &str) -> Result<Self, IdentifierError> {
                Self::from_str(chain_id)
            }
            /// Get a reference to the underlying string.
            pub fn as_str(&self) -> &str {
                &self.id
            }
            pub fn split_chain_id(&self) -> Result<(&str, u64), IdentifierError> {
                parse_chain_id_string(self.as_str())
            }
            /// Extract the revision number from the chain identifier
            pub fn revision_number(&self) -> u64 {
                self.revision_number
            }
            /// Increases `ChainId`s revision number by one.
            /// Fails if the chain identifier is not in
            /// `{chain_name}-{revision_number}` format or
            /// the revision number overflows.
            ///
            /// ```
            /// use ibc_core_host_types::identifiers::ChainId;
            ///
            /// let mut chain_id = ChainId::new("chainA-1").unwrap();
            /// assert!(chain_id.increment_revision_number().is_ok());
            /// assert_eq!(chain_id.revision_number(), 2);
            ///
            /// let mut chain_id = ChainId::new(&format!("chainA-{}", u64::MAX)).unwrap();
            /// assert!(chain_id.increment_revision_number().is_err());
            /// assert_eq!(chain_id.revision_number(), u64::MAX);
            /// ```
            pub fn increment_revision_number(&mut self) -> Result<(), IdentifierError> {
                let (chain_name, _) = self.split_chain_id()?;
                let inc_revision_number = self
                    .revision_number
                    .checked_add(1)
                    .ok_or(IdentifierError::RevisionNumberOverflow)?;
                self
                    .id = {
                    let res = ::alloc::fmt::format(
                        format_args!("{0}-{1}", chain_name, inc_revision_number),
                    );
                    res
                };
                self.revision_number = inc_revision_number;
                Ok(())
            }
            /// A convenient method to check if the `ChainId` forms a valid identifier
            /// with the desired min/max length. However, ICS-24 does not specify a
            /// certain min or max lengths for chain identifiers.
            pub fn validate_length(
                &self,
                min_length: u64,
                max_length: u64,
            ) -> Result<(), IdentifierError> {
                match self.split_chain_id() {
                    Ok((chain_name, _)) => {
                        validate_prefix_length(chain_name, min_length, max_length)
                    }
                    _ => validate_identifier_length(&self.id, min_length, max_length),
                }
            }
        }
        /// Construct a `ChainId` from a string literal only if it forms a valid
        /// identifier.
        impl FromStr for ChainId {
            type Err = IdentifierError;
            fn from_str(id: &str) -> Result<Self, Self::Err> {
                validate_identifier_chars(id)?;
                match parse_chain_id_string(id) {
                    Ok((chain_name, revision_number)) => {
                        validate_prefix_length(chain_name, 1, 64)?;
                        Ok(Self {
                            id: id.into(),
                            revision_number,
                        })
                    }
                    _ => {
                        validate_identifier_length(id, 1, 64)?;
                        Ok(Self {
                            id: id.into(),
                            revision_number: 0,
                        })
                    }
                }
            }
        }
        impl From<ChainId> for String {
            fn from(chain_id: ChainId) -> String {
                chain_id.id
            }
        }
        impl Display for ChainId {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
                f.write_fmt(format_args!("{0}", self.id))
            }
        }
        /// Parses a string intended to represent a `ChainId` and, if successful,
        /// returns a tuple containing the chain name and revision number.
        fn parse_chain_id_string(
            chain_id_str: &str,
        ) -> Result<(&str, u64), IdentifierError> {
            chain_id_str
                .rsplit_once('-')
                .filter(|(_, rev_number_str)| {
                    rev_number_str.as_bytes().first() != Some(&b'0')
                        || rev_number_str.len() == 1
                })
                .and_then(|(chain_name, rev_number_str)| {
                    rev_number_str
                        .parse()
                        .ok()
                        .map(|revision_number| (chain_name, revision_number))
                })
                .ok_or(IdentifierError::UnformattedRevisionNumber {
                    chain_id: chain_id_str.to_string(),
                })
        }
    }
    mod channel_id {
        use core::fmt::{Debug, Display, Error as FmtError, Formatter};
        use core::str::FromStr;
        use derive_more::Into;
        use ibc_primitives::prelude::*;
        use crate::error::IdentifierError;
        use crate::validate::validate_channel_identifier;
        const CHANNEL_ID_PREFIX: &str = "channel";
        pub struct ChannelId(String);
        #[automatically_derived]
        impl ::core::clone::Clone for ChannelId {
            #[inline]
            fn clone(&self) -> ChannelId {
                ChannelId(::core::clone::Clone::clone(&self.0))
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for ChannelId {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "ChannelId",
                    &&self.0,
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for ChannelId {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for ChannelId {
            #[inline]
            fn eq(&self, other: &ChannelId) -> bool {
                self.0 == other.0
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for ChannelId {}
        #[automatically_derived]
        impl ::core::cmp::Eq for ChannelId {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<String>;
            }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialOrd for ChannelId {
            #[inline]
            fn partial_cmp(
                &self,
                other: &ChannelId,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Ord for ChannelId {
            #[inline]
            fn cmp(&self, other: &ChannelId) -> ::core::cmp::Ordering {
                ::core::cmp::Ord::cmp(&self.0, &other.0)
            }
        }
        #[automatically_derived]
        impl ::core::hash::Hash for ChannelId {
            #[inline]
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                ::core::hash::Hash::hash(&self.0, state)
            }
        }
        #[automatically_derived]
        impl ::core::convert::From<ChannelId> for (String) {
            #[inline]
            fn from(original: ChannelId) -> Self {
                (original.0)
            }
        }
        impl ChannelId {
            /// Builds a new channel identifier. Like client and connection identifiers, channel ids are
            /// deterministically formed from two elements: a prefix `prefix`, and a monotonically
            /// increasing `counter`, separated by a dash "-".
            /// The prefix is currently determined statically (see `ChannelId::prefix()`) so this method
            /// accepts a single argument, the `counter`.
            ///
            /// ```
            /// # use ibc_core_host_types::identifiers::ChannelId;
            /// let chan_id = ChannelId::new(27);
            /// assert_eq!(chan_id.to_string(), "channel-27");
            /// ```
            pub fn new(identifier: u64) -> Self {
                let id = {
                    let res = ::alloc::fmt::format(
                        format_args!("{0}-{1}", Self::prefix(), identifier),
                    );
                    res
                };
                Self(id)
            }
            /// Returns the static prefix to be used across all channel identifiers.
            pub fn prefix() -> &'static str {
                CHANNEL_ID_PREFIX
            }
            /// Get this identifier as a borrowed `&str`
            pub fn as_str(&self) -> &str {
                &self.0
            }
            /// Get this identifier as a borrowed byte slice
            pub fn as_bytes(&self) -> &[u8] {
                self.0.as_bytes()
            }
        }
        /// This implementation provides a `to_string` method.
        impl Display for ChannelId {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
                f.write_fmt(format_args!("{0}", self.0))
            }
        }
        impl FromStr for ChannelId {
            type Err = IdentifierError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                validate_channel_identifier(s).map(|_| Self(s.to_string()))
            }
        }
        impl AsRef<str> for ChannelId {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
        impl Default for ChannelId {
            fn default() -> Self {
                Self::new(0)
            }
        }
        /// Equality check against string literal (satisfies &ChannelId == &str).
        /// ```
        /// use core::str::FromStr;
        /// use ibc_core_host_types::identifiers::ChannelId;
        /// let channel_id = ChannelId::from_str("channelId-0");
        /// assert!(channel_id.is_ok());
        /// channel_id.map(|id| {assert_eq!(&id, "channelId-0")});
        /// ```
        impl PartialEq<str> for ChannelId {
            fn eq(&self, other: &str) -> bool {
                self.as_str().eq(other)
            }
        }
    }
    mod client_id {
        use core::fmt::{Debug, Display, Error as FmtError, Formatter};
        use core::str::FromStr;
        use derive_more::Into;
        use ibc_primitives::prelude::*;
        use super::ClientType;
        use crate::error::IdentifierError;
        use crate::validate::{validate_client_identifier, validate_client_type};
        pub struct ClientId(String);
        #[automatically_derived]
        impl ::core::clone::Clone for ClientId {
            #[inline]
            fn clone(&self) -> ClientId {
                ClientId(::core::clone::Clone::clone(&self.0))
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for ClientId {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "ClientId",
                    &&self.0,
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for ClientId {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for ClientId {
            #[inline]
            fn eq(&self, other: &ClientId) -> bool {
                self.0 == other.0
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for ClientId {}
        #[automatically_derived]
        impl ::core::cmp::Eq for ClientId {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<String>;
            }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialOrd for ClientId {
            #[inline]
            fn partial_cmp(
                &self,
                other: &ClientId,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Ord for ClientId {
            #[inline]
            fn cmp(&self, other: &ClientId) -> ::core::cmp::Ordering {
                ::core::cmp::Ord::cmp(&self.0, &other.0)
            }
        }
        #[automatically_derived]
        impl ::core::hash::Hash for ClientId {
            #[inline]
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                ::core::hash::Hash::hash(&self.0, state)
            }
        }
        #[automatically_derived]
        impl ::core::convert::From<ClientId> for (String) {
            #[inline]
            fn from(original: ClientId) -> Self {
                (original.0)
            }
        }
        impl ClientId {
            /// Builds a new client identifier. Client identifiers are deterministically formed from two
            /// elements: a prefix derived from the client type `ctype`, and a monotonically increasing
            /// `counter`; these are separated by a dash "-".
            ///
            /// ```
            /// # use ibc_core_host_types::identifiers::ClientId;
            /// # use ibc_core_host_types::identifiers::ClientType;
            /// # use std::str::FromStr;
            /// let tm_client_id = ClientId::new(ClientType::from_str("07-tendermint").unwrap(), 0);
            /// assert!(tm_client_id.is_ok());
            /// tm_client_id.map(|id| { assert_eq!(&id, "07-tendermint-0") });
            /// ```
            pub fn new(
                client_type: ClientType,
                counter: u64,
            ) -> Result<Self, IdentifierError> {
                let prefix = client_type.as_str().trim();
                validate_client_type(prefix)?;
                let id = {
                    let res = ::alloc::fmt::format(
                        format_args!("{0}-{1}", prefix, counter),
                    );
                    res
                };
                Self::from_str(id.as_str())
            }
            /// Get this identifier as a borrowed `&str`
            pub fn as_str(&self) -> &str {
                &self.0
            }
            /// Get this identifier as a borrowed byte slice
            pub fn as_bytes(&self) -> &[u8] {
                self.0.as_bytes()
            }
        }
        /// This implementation provides a `to_string` method.
        impl Display for ClientId {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
                f.write_fmt(format_args!("{0}", self.0))
            }
        }
        impl FromStr for ClientId {
            type Err = IdentifierError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                validate_client_identifier(s).map(|_| Self(s.to_string()))
            }
        }
        impl Default for ClientId {
            fn default() -> Self {
                Self::from_str("07-tendermint-0")
                    .expect("Never fails because we use a valid client id")
            }
        }
        /// Equality check against string literal (satisfies &ClientId == &str).
        /// ```
        /// use core::str::FromStr;
        /// use ibc_core_host_types::identifiers::ClientId;
        /// let client_id = ClientId::from_str("clientidtwo");
        /// assert!(client_id.is_ok());
        /// client_id.map(|id| {assert_eq!(&id, "clientidtwo")});
        /// ```
        impl PartialEq<str> for ClientId {
            fn eq(&self, other: &str) -> bool {
                self.as_str().eq(other)
            }
        }
    }
    mod client_type {
        //! Defines the `ClientType` format, typically used in chain IDs.
        use core::fmt::{Display, Error as FmtError, Formatter};
        use core::str::FromStr;
        use ibc_primitives::prelude::*;
        use crate::error::IdentifierError;
        use crate::validate::validate_client_type;
        /// Type of the client, depending on the specific consensus algorithm.
        pub struct ClientType(String);
        #[automatically_derived]
        impl ::core::clone::Clone for ClientType {
            #[inline]
            fn clone(&self) -> ClientType {
                ClientType(::core::clone::Clone::clone(&self.0))
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for ClientType {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "ClientType",
                    &&self.0,
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for ClientType {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for ClientType {
            #[inline]
            fn eq(&self, other: &ClientType) -> bool {
                self.0 == other.0
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for ClientType {}
        #[automatically_derived]
        impl ::core::cmp::Eq for ClientType {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<String>;
            }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialOrd for ClientType {
            #[inline]
            fn partial_cmp(
                &self,
                other: &ClientType,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Ord for ClientType {
            #[inline]
            fn cmp(&self, other: &ClientType) -> ::core::cmp::Ordering {
                ::core::cmp::Ord::cmp(&self.0, &other.0)
            }
        }
        #[automatically_derived]
        impl ::core::hash::Hash for ClientType {
            #[inline]
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                ::core::hash::Hash::hash(&self.0, state)
            }
        }
        impl ClientType {
            /// Constructs a new `ClientType` from the given `String` if it ends with a valid client identifier.
            pub fn new(s: &str) -> Result<Self, IdentifierError> {
                let s_trim = s.trim();
                validate_client_type(s_trim)?;
                Ok(Self(s_trim.to_string()))
            }
            /// Yields this identifier as a borrowed `&str`
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl FromStr for ClientType {
            type Err = IdentifierError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::new(s)
            }
        }
        impl Display for ClientType {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
                f.write_fmt(format_args!("ClientType({0})", self.0))
            }
        }
    }
    mod connection_id {
        use core::fmt::{Display, Error as FmtError, Formatter};
        use core::str::FromStr;
        use derive_more::Into;
        use ibc_primitives::prelude::*;
        use crate::error::IdentifierError;
        use crate::validate::validate_connection_identifier;
        const CONNECTION_ID_PREFIX: &str = "connection";
        pub struct ConnectionId(String);
        #[automatically_derived]
        impl ::core::clone::Clone for ConnectionId {
            #[inline]
            fn clone(&self) -> ConnectionId {
                ConnectionId(::core::clone::Clone::clone(&self.0))
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for ConnectionId {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "ConnectionId",
                    &&self.0,
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for ConnectionId {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for ConnectionId {
            #[inline]
            fn eq(&self, other: &ConnectionId) -> bool {
                self.0 == other.0
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for ConnectionId {}
        #[automatically_derived]
        impl ::core::cmp::Eq for ConnectionId {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<String>;
            }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialOrd for ConnectionId {
            #[inline]
            fn partial_cmp(
                &self,
                other: &ConnectionId,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Ord for ConnectionId {
            #[inline]
            fn cmp(&self, other: &ConnectionId) -> ::core::cmp::Ordering {
                ::core::cmp::Ord::cmp(&self.0, &other.0)
            }
        }
        #[automatically_derived]
        impl ::core::hash::Hash for ConnectionId {
            #[inline]
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                ::core::hash::Hash::hash(&self.0, state)
            }
        }
        #[automatically_derived]
        impl ::core::convert::From<ConnectionId> for (String) {
            #[inline]
            fn from(original: ConnectionId) -> Self {
                (original.0)
            }
        }
        impl ConnectionId {
            /// Builds a new connection identifier. Connection identifiers are deterministically formed from
            /// two elements: a prefix `prefix`, and a monotonically increasing `counter`; these are
            /// separated by a dash "-". The prefix is currently determined statically (see
            /// `ConnectionId::prefix()`) so this method accepts a single argument, the `counter`.
            ///
            /// ```
            /// # use ibc_core_host_types::identifiers::ConnectionId;
            /// let conn_id = ConnectionId::new(11);
            /// assert_eq!(&conn_id, "connection-11");
            /// ```
            pub fn new(identifier: u64) -> Self {
                let id = {
                    let res = ::alloc::fmt::format(
                        format_args!("{0}-{1}", Self::prefix(), identifier),
                    );
                    res
                };
                Self(id)
            }
            /// Returns the static prefix to be used across all connection identifiers.
            pub fn prefix() -> &'static str {
                CONNECTION_ID_PREFIX
            }
            /// Get this identifier as a borrowed `&str`
            pub fn as_str(&self) -> &str {
                &self.0
            }
            /// Get this identifier as a borrowed byte slice
            pub fn as_bytes(&self) -> &[u8] {
                self.0.as_bytes()
            }
        }
        /// This implementation provides a `to_string` method.
        impl Display for ConnectionId {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
                f.write_fmt(format_args!("{0}", self.0))
            }
        }
        impl FromStr for ConnectionId {
            type Err = IdentifierError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                validate_connection_identifier(s).map(|_| Self(s.to_string()))
            }
        }
        impl Default for ConnectionId {
            fn default() -> Self {
                Self::new(0)
            }
        }
        /// Equality check against string literal (satisfies &ConnectionId == &str).
        /// ```
        /// use core::str::FromStr;
        /// use ibc_core_host_types::identifiers::ConnectionId;
        /// let conn_id = ConnectionId::from_str("connectionId-0");
        /// assert!(conn_id.is_ok());
        /// conn_id.map(|id| {assert_eq!(&id, "connectionId-0")});
        /// ```
        impl PartialEq<str> for ConnectionId {
            fn eq(&self, other: &str) -> bool {
                self.as_str().eq(other)
            }
        }
    }
    mod port_id {
        use core::fmt::{Display, Error as FmtError, Formatter};
        use core::str::FromStr;
        use derive_more::Into;
        use ibc_primitives::prelude::*;
        use crate::error::IdentifierError;
        use crate::validate::validate_port_identifier;
        const TRANSFER_PORT_ID: &str = "transfer";
        pub struct PortId(String);
        #[automatically_derived]
        impl ::core::clone::Clone for PortId {
            #[inline]
            fn clone(&self) -> PortId {
                PortId(::core::clone::Clone::clone(&self.0))
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for PortId {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "PortId", &&self.0)
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for PortId {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for PortId {
            #[inline]
            fn eq(&self, other: &PortId) -> bool {
                self.0 == other.0
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for PortId {}
        #[automatically_derived]
        impl ::core::cmp::Eq for PortId {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<String>;
            }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialOrd for PortId {
            #[inline]
            fn partial_cmp(
                &self,
                other: &PortId,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Ord for PortId {
            #[inline]
            fn cmp(&self, other: &PortId) -> ::core::cmp::Ordering {
                ::core::cmp::Ord::cmp(&self.0, &other.0)
            }
        }
        #[automatically_derived]
        impl ::core::hash::Hash for PortId {
            #[inline]
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                ::core::hash::Hash::hash(&self.0, state)
            }
        }
        #[automatically_derived]
        impl ::core::convert::From<PortId> for (String) {
            #[inline]
            fn from(original: PortId) -> Self {
                (original.0)
            }
        }
        impl PortId {
            pub fn new(id: String) -> Result<Self, IdentifierError> {
                Self::from_str(&id)
            }
            /// Infallible creation of the well-known transfer port
            pub fn transfer() -> Self {
                Self(TRANSFER_PORT_ID.to_string())
            }
            /// Get this identifier as a borrowed `&str`
            pub fn as_str(&self) -> &str {
                &self.0
            }
            /// Get this identifier as a borrowed byte slice
            pub fn as_bytes(&self) -> &[u8] {
                self.0.as_bytes()
            }
            pub fn validate(&self) -> Result<(), IdentifierError> {
                validate_port_identifier(self.as_str())
            }
        }
        /// This implementation provides a `to_string` method.
        impl Display for PortId {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
                f.write_fmt(format_args!("{0}", self.0))
            }
        }
        impl FromStr for PortId {
            type Err = IdentifierError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                validate_port_identifier(s).map(|_| Self(s.to_string()))
            }
        }
        impl AsRef<str> for PortId {
            fn as_ref(&self) -> &str {
                self.0.as_str()
            }
        }
    }
    mod sequence {
        use ibc_primitives::prelude::*;
        use ibc_primitives::ToVec;
        use crate::error::IdentifierError;
        /// The sequence number of a packet enforces ordering among packets from the same source.
        pub struct Sequence(u64);
        #[automatically_derived]
        impl ::core::marker::Copy for Sequence {}
        #[automatically_derived]
        impl ::core::clone::Clone for Sequence {
            #[inline]
            fn clone(&self) -> Sequence {
                let _: ::core::clone::AssertParamIsClone<u64>;
                *self
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Sequence {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Sequence",
                    &&self.0,
                )
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for Sequence {
            #[inline]
            fn default() -> Sequence {
                Sequence(::core::default::Default::default())
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Sequence {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Sequence {
            #[inline]
            fn eq(&self, other: &Sequence) -> bool {
                self.0 == other.0
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for Sequence {}
        #[automatically_derived]
        impl ::core::cmp::Eq for Sequence {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<u64>;
            }
        }
        #[automatically_derived]
        impl ::core::hash::Hash for Sequence {
            #[inline]
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                ::core::hash::Hash::hash(&self.0, state)
            }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialOrd for Sequence {
            #[inline]
            fn partial_cmp(
                &self,
                other: &Sequence,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Ord for Sequence {
            #[inline]
            fn cmp(&self, other: &Sequence) -> ::core::cmp::Ordering {
                ::core::cmp::Ord::cmp(&self.0, &other.0)
            }
        }
        impl core::str::FromStr for Sequence {
            type Err = IdentifierError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(
                    Self::from(
                        s
                            .parse::<u64>()
                            .map_err(|e| {
                                IdentifierError::InvalidStringAsSequence {
                                    value: s.to_string(),
                                    reason: e.to_string(),
                                }
                            })?,
                    ),
                )
            }
        }
        impl Sequence {
            /// Gives the sequence number.
            pub fn value(&self) -> u64 {
                self.0
            }
            /// Returns `true` if the sequence number is zero.
            pub fn is_zero(&self) -> bool {
                self.0 == 0
            }
            /// Increments the sequence number by one.
            pub fn increment(&self) -> Sequence {
                Sequence(self.0 + 1)
            }
            /// Encodes the sequence number into a `Vec<u8>` using
            /// `prost::Message::encode_to_vec`.
            pub fn to_vec(&self) -> Vec<u8> {
                self.0.to_vec()
            }
        }
        impl From<u64> for Sequence {
            fn from(seq: u64) -> Self {
                Sequence(seq)
            }
        }
        impl From<Sequence> for u64 {
            fn from(s: Sequence) -> u64 {
                s.0
            }
        }
        impl core::fmt::Display for Sequence {
            fn fmt(
                &self,
                f: &mut core::fmt::Formatter<'_>,
            ) -> Result<(), core::fmt::Error> {
                f.write_fmt(format_args!("{0}", self.0))
            }
        }
    }
    pub use chain_id::ChainId;
    pub use channel_id::ChannelId;
    pub use client_id::ClientId;
    pub use client_type::ClientType;
    pub use connection_id::ConnectionId;
    pub use port_id::PortId;
    pub use sequence::Sequence;
}
pub mod path {
    //! Defines all store paths used by IBC
    /// Path-space as listed in ICS-024
    /// https://github.com/cosmos/ibc/tree/master/spec/core/ics-024-host-requirements#path-space
    /// Some of these are implemented in other ICSs, but ICS-024 has a nice summary table.
    ///
    use core::str::FromStr;
    use derive_more::{Display, From};
    use ibc_primitives::prelude::*;
    use crate::identifiers::{ChannelId, ClientId, ConnectionId, PortId, Sequence};
    /// ABCI client upgrade keys
    /// - The key identifying the upgraded IBC state within the upgrade sub-store
    const UPGRADED_IBC_STATE: &str = "upgradedIBCState";
    ///- The key identifying the upgraded client state
    const UPGRADED_CLIENT_STATE: &str = "upgradedClient";
    /// - The key identifying the upgraded consensus state
    const UPGRADED_CLIENT_CONSENSUS_STATE: &str = "upgradedConsState";
    /// The Path enum abstracts out the different sub-paths.
    pub enum Path {
        ClientState(ClientStatePath),
        ClientConsensusState(ClientConsensusStatePath),
        ClientConnection(ClientConnectionPath),
        Connection(ConnectionPath),
        Ports(PortPath),
        ChannelEnd(ChannelEndPath),
        SeqSend(SeqSendPath),
        SeqRecv(SeqRecvPath),
        SeqAck(SeqAckPath),
        Commitment(CommitmentPath),
        Ack(AckPath),
        Receipt(ReceiptPath),
        UpgradeClient(UpgradeClientPath),
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Path {
        #[inline]
        fn clone(&self) -> Path {
            match self {
                Path::ClientState(__self_0) => {
                    Path::ClientState(::core::clone::Clone::clone(__self_0))
                }
                Path::ClientConsensusState(__self_0) => {
                    Path::ClientConsensusState(::core::clone::Clone::clone(__self_0))
                }
                Path::ClientConnection(__self_0) => {
                    Path::ClientConnection(::core::clone::Clone::clone(__self_0))
                }
                Path::Connection(__self_0) => {
                    Path::Connection(::core::clone::Clone::clone(__self_0))
                }
                Path::Ports(__self_0) => {
                    Path::Ports(::core::clone::Clone::clone(__self_0))
                }
                Path::ChannelEnd(__self_0) => {
                    Path::ChannelEnd(::core::clone::Clone::clone(__self_0))
                }
                Path::SeqSend(__self_0) => {
                    Path::SeqSend(::core::clone::Clone::clone(__self_0))
                }
                Path::SeqRecv(__self_0) => {
                    Path::SeqRecv(::core::clone::Clone::clone(__self_0))
                }
                Path::SeqAck(__self_0) => {
                    Path::SeqAck(::core::clone::Clone::clone(__self_0))
                }
                Path::Commitment(__self_0) => {
                    Path::Commitment(::core::clone::Clone::clone(__self_0))
                }
                Path::Ack(__self_0) => Path::Ack(::core::clone::Clone::clone(__self_0)),
                Path::Receipt(__self_0) => {
                    Path::Receipt(::core::clone::Clone::clone(__self_0))
                }
                Path::UpgradeClient(__self_0) => {
                    Path::UpgradeClient(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Path {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                Path::ClientState(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "ClientState",
                        &__self_0,
                    )
                }
                Path::ClientConsensusState(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "ClientConsensusState",
                        &__self_0,
                    )
                }
                Path::ClientConnection(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "ClientConnection",
                        &__self_0,
                    )
                }
                Path::Connection(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Connection",
                        &__self_0,
                    )
                }
                Path::Ports(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Ports",
                        &__self_0,
                    )
                }
                Path::ChannelEnd(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "ChannelEnd",
                        &__self_0,
                    )
                }
                Path::SeqSend(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "SeqSend",
                        &__self_0,
                    )
                }
                Path::SeqRecv(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "SeqRecv",
                        &__self_0,
                    )
                }
                Path::SeqAck(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "SeqAck",
                        &__self_0,
                    )
                }
                Path::Commitment(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Commitment",
                        &__self_0,
                    )
                }
                Path::Ack(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Ack",
                        &__self_0,
                    )
                }
                Path::Receipt(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Receipt",
                        &__self_0,
                    )
                }
                Path::UpgradeClient(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "UpgradeClient",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Path {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Path {
        #[inline]
        fn eq(&self, other: &Path) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
                && match (self, other) {
                    (Path::ClientState(__self_0), Path::ClientState(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (
                        Path::ClientConsensusState(__self_0),
                        Path::ClientConsensusState(__arg1_0),
                    ) => *__self_0 == *__arg1_0,
                    (
                        Path::ClientConnection(__self_0),
                        Path::ClientConnection(__arg1_0),
                    ) => *__self_0 == *__arg1_0,
                    (Path::Connection(__self_0), Path::Connection(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (Path::Ports(__self_0), Path::Ports(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (Path::ChannelEnd(__self_0), Path::ChannelEnd(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (Path::SeqSend(__self_0), Path::SeqSend(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (Path::SeqRecv(__self_0), Path::SeqRecv(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (Path::SeqAck(__self_0), Path::SeqAck(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (Path::Commitment(__self_0), Path::Commitment(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (Path::Ack(__self_0), Path::Ack(__arg1_0)) => *__self_0 == *__arg1_0,
                    (Path::Receipt(__self_0), Path::Receipt(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (Path::UpgradeClient(__self_0), Path::UpgradeClient(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    _ => unsafe { ::core::intrinsics::unreachable() }
                }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for Path {}
    #[automatically_derived]
    impl ::core::cmp::Eq for Path {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<ClientStatePath>;
            let _: ::core::cmp::AssertParamIsEq<ClientConsensusStatePath>;
            let _: ::core::cmp::AssertParamIsEq<ClientConnectionPath>;
            let _: ::core::cmp::AssertParamIsEq<ConnectionPath>;
            let _: ::core::cmp::AssertParamIsEq<PortPath>;
            let _: ::core::cmp::AssertParamIsEq<ChannelEndPath>;
            let _: ::core::cmp::AssertParamIsEq<SeqSendPath>;
            let _: ::core::cmp::AssertParamIsEq<SeqRecvPath>;
            let _: ::core::cmp::AssertParamIsEq<SeqAckPath>;
            let _: ::core::cmp::AssertParamIsEq<CommitmentPath>;
            let _: ::core::cmp::AssertParamIsEq<AckPath>;
            let _: ::core::cmp::AssertParamIsEq<ReceiptPath>;
            let _: ::core::cmp::AssertParamIsEq<UpgradeClientPath>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for Path {
        #[inline]
        fn partial_cmp(
            &self,
            other: &Path,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            match (self, other) {
                (Path::ClientState(__self_0), Path::ClientState(__arg1_0)) => {
                    ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0)
                }
                (
                    Path::ClientConsensusState(__self_0),
                    Path::ClientConsensusState(__arg1_0),
                ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
                (Path::ClientConnection(__self_0), Path::ClientConnection(__arg1_0)) => {
                    ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0)
                }
                (Path::Connection(__self_0), Path::Connection(__arg1_0)) => {
                    ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0)
                }
                (Path::Ports(__self_0), Path::Ports(__arg1_0)) => {
                    ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0)
                }
                (Path::ChannelEnd(__self_0), Path::ChannelEnd(__arg1_0)) => {
                    ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0)
                }
                (Path::SeqSend(__self_0), Path::SeqSend(__arg1_0)) => {
                    ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0)
                }
                (Path::SeqRecv(__self_0), Path::SeqRecv(__arg1_0)) => {
                    ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0)
                }
                (Path::SeqAck(__self_0), Path::SeqAck(__arg1_0)) => {
                    ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0)
                }
                (Path::Commitment(__self_0), Path::Commitment(__arg1_0)) => {
                    ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0)
                }
                (Path::Ack(__self_0), Path::Ack(__arg1_0)) => {
                    ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0)
                }
                (Path::Receipt(__self_0), Path::Receipt(__arg1_0)) => {
                    ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0)
                }
                (Path::UpgradeClient(__self_0), Path::UpgradeClient(__arg1_0)) => {
                    ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0)
                }
                _ => ::core::cmp::PartialOrd::partial_cmp(&__self_tag, &__arg1_tag),
            }
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for Path {
        #[inline]
        fn cmp(&self, other: &Path) -> ::core::cmp::Ordering {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            match ::core::cmp::Ord::cmp(&__self_tag, &__arg1_tag) {
                ::core::cmp::Ordering::Equal => {
                    match (self, other) {
                        (Path::ClientState(__self_0), Path::ClientState(__arg1_0)) => {
                            ::core::cmp::Ord::cmp(__self_0, __arg1_0)
                        }
                        (
                            Path::ClientConsensusState(__self_0),
                            Path::ClientConsensusState(__arg1_0),
                        ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                        (
                            Path::ClientConnection(__self_0),
                            Path::ClientConnection(__arg1_0),
                        ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                        (Path::Connection(__self_0), Path::Connection(__arg1_0)) => {
                            ::core::cmp::Ord::cmp(__self_0, __arg1_0)
                        }
                        (Path::Ports(__self_0), Path::Ports(__arg1_0)) => {
                            ::core::cmp::Ord::cmp(__self_0, __arg1_0)
                        }
                        (Path::ChannelEnd(__self_0), Path::ChannelEnd(__arg1_0)) => {
                            ::core::cmp::Ord::cmp(__self_0, __arg1_0)
                        }
                        (Path::SeqSend(__self_0), Path::SeqSend(__arg1_0)) => {
                            ::core::cmp::Ord::cmp(__self_0, __arg1_0)
                        }
                        (Path::SeqRecv(__self_0), Path::SeqRecv(__arg1_0)) => {
                            ::core::cmp::Ord::cmp(__self_0, __arg1_0)
                        }
                        (Path::SeqAck(__self_0), Path::SeqAck(__arg1_0)) => {
                            ::core::cmp::Ord::cmp(__self_0, __arg1_0)
                        }
                        (Path::Commitment(__self_0), Path::Commitment(__arg1_0)) => {
                            ::core::cmp::Ord::cmp(__self_0, __arg1_0)
                        }
                        (Path::Ack(__self_0), Path::Ack(__arg1_0)) => {
                            ::core::cmp::Ord::cmp(__self_0, __arg1_0)
                        }
                        (Path::Receipt(__self_0), Path::Receipt(__arg1_0)) => {
                            ::core::cmp::Ord::cmp(__self_0, __arg1_0)
                        }
                        (
                            Path::UpgradeClient(__self_0),
                            Path::UpgradeClient(__arg1_0),
                        ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                        _ => unsafe { ::core::intrinsics::unreachable() }
                    }
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for Path {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_tag, state);
            match self {
                Path::ClientState(__self_0) => ::core::hash::Hash::hash(__self_0, state),
                Path::ClientConsensusState(__self_0) => {
                    ::core::hash::Hash::hash(__self_0, state)
                }
                Path::ClientConnection(__self_0) => {
                    ::core::hash::Hash::hash(__self_0, state)
                }
                Path::Connection(__self_0) => ::core::hash::Hash::hash(__self_0, state),
                Path::Ports(__self_0) => ::core::hash::Hash::hash(__self_0, state),
                Path::ChannelEnd(__self_0) => ::core::hash::Hash::hash(__self_0, state),
                Path::SeqSend(__self_0) => ::core::hash::Hash::hash(__self_0, state),
                Path::SeqRecv(__self_0) => ::core::hash::Hash::hash(__self_0, state),
                Path::SeqAck(__self_0) => ::core::hash::Hash::hash(__self_0, state),
                Path::Commitment(__self_0) => ::core::hash::Hash::hash(__self_0, state),
                Path::Ack(__self_0) => ::core::hash::Hash::hash(__self_0, state),
                Path::Receipt(__self_0) => ::core::hash::Hash::hash(__self_0, state),
                Path::UpgradeClient(__self_0) => {
                    ::core::hash::Hash::hash(__self_0, state)
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::convert::From<(ConnectionPath)> for Path {
        #[inline]
        fn from(original: (ConnectionPath)) -> Path {
            Path::Connection(original)
        }
    }
    #[automatically_derived]
    impl ::core::convert::From<(UpgradeClientPath)> for Path {
        #[inline]
        fn from(original: (UpgradeClientPath)) -> Path {
            Path::UpgradeClient(original)
        }
    }
    #[automatically_derived]
    impl ::core::convert::From<(AckPath)> for Path {
        #[inline]
        fn from(original: (AckPath)) -> Path {
            Path::Ack(original)
        }
    }
    #[automatically_derived]
    impl ::core::convert::From<(ClientConsensusStatePath)> for Path {
        #[inline]
        fn from(original: (ClientConsensusStatePath)) -> Path {
            Path::ClientConsensusState(original)
        }
    }
    #[automatically_derived]
    impl ::core::convert::From<(ClientStatePath)> for Path {
        #[inline]
        fn from(original: (ClientStatePath)) -> Path {
            Path::ClientState(original)
        }
    }
    #[automatically_derived]
    impl ::core::convert::From<(ClientConnectionPath)> for Path {
        #[inline]
        fn from(original: (ClientConnectionPath)) -> Path {
            Path::ClientConnection(original)
        }
    }
    #[automatically_derived]
    impl ::core::convert::From<(SeqSendPath)> for Path {
        #[inline]
        fn from(original: (SeqSendPath)) -> Path {
            Path::SeqSend(original)
        }
    }
    #[automatically_derived]
    impl ::core::convert::From<(SeqAckPath)> for Path {
        #[inline]
        fn from(original: (SeqAckPath)) -> Path {
            Path::SeqAck(original)
        }
    }
    #[automatically_derived]
    impl ::core::convert::From<(ChannelEndPath)> for Path {
        #[inline]
        fn from(original: (ChannelEndPath)) -> Path {
            Path::ChannelEnd(original)
        }
    }
    #[automatically_derived]
    impl ::core::convert::From<(CommitmentPath)> for Path {
        #[inline]
        fn from(original: (CommitmentPath)) -> Path {
            Path::Commitment(original)
        }
    }
    #[automatically_derived]
    impl ::core::convert::From<(PortPath)> for Path {
        #[inline]
        fn from(original: (PortPath)) -> Path {
            Path::Ports(original)
        }
    }
    #[automatically_derived]
    impl ::core::convert::From<(ReceiptPath)> for Path {
        #[inline]
        fn from(original: (ReceiptPath)) -> Path {
            Path::Receipt(original)
        }
    }
    #[automatically_derived]
    impl ::core::convert::From<(SeqRecvPath)> for Path {
        #[inline]
        fn from(original: (SeqRecvPath)) -> Path {
            Path::SeqRecv(original)
        }
    }
    impl ::core::fmt::Display for Path {
        #[allow(unused_variables)]
        #[inline]
        fn fmt(
            &self,
            _derive_more_display_formatter: &mut ::core::fmt::Formatter,
        ) -> ::core::fmt::Result {
            match self {
                Path::ClientState(_0) => {
                    ::core::fmt::Display::fmt(_0, _derive_more_display_formatter)
                }
                Path::ClientConsensusState(_0) => {
                    ::core::fmt::Display::fmt(_0, _derive_more_display_formatter)
                }
                Path::ClientConnection(_0) => {
                    ::core::fmt::Display::fmt(_0, _derive_more_display_formatter)
                }
                Path::Connection(_0) => {
                    ::core::fmt::Display::fmt(_0, _derive_more_display_formatter)
                }
                Path::Ports(_0) => {
                    ::core::fmt::Display::fmt(_0, _derive_more_display_formatter)
                }
                Path::ChannelEnd(_0) => {
                    ::core::fmt::Display::fmt(_0, _derive_more_display_formatter)
                }
                Path::SeqSend(_0) => {
                    ::core::fmt::Display::fmt(_0, _derive_more_display_formatter)
                }
                Path::SeqRecv(_0) => {
                    ::core::fmt::Display::fmt(_0, _derive_more_display_formatter)
                }
                Path::SeqAck(_0) => {
                    ::core::fmt::Display::fmt(_0, _derive_more_display_formatter)
                }
                Path::Commitment(_0) => {
                    ::core::fmt::Display::fmt(_0, _derive_more_display_formatter)
                }
                Path::Ack(_0) => {
                    ::core::fmt::Display::fmt(_0, _derive_more_display_formatter)
                }
                Path::Receipt(_0) => {
                    ::core::fmt::Display::fmt(_0, _derive_more_display_formatter)
                }
                Path::UpgradeClient(_0) => {
                    ::core::fmt::Display::fmt(_0, _derive_more_display_formatter)
                }
                _ => Ok(()),
            }
        }
    }
    #[display(fmt = "clients/{_0}/clientState")]
    pub struct ClientStatePath(pub ClientId);
    #[automatically_derived]
    impl ::core::clone::Clone for ClientStatePath {
        #[inline]
        fn clone(&self) -> ClientStatePath {
            ClientStatePath(::core::clone::Clone::clone(&self.0))
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ClientStatePath {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(
                f,
                "ClientStatePath",
                &&self.0,
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ClientStatePath {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ClientStatePath {
        #[inline]
        fn eq(&self, other: &ClientStatePath) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for ClientStatePath {}
    #[automatically_derived]
    impl ::core::cmp::Eq for ClientStatePath {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<ClientId>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ClientStatePath {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ClientStatePath,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for ClientStatePath {
        #[inline]
        fn cmp(&self, other: &ClientStatePath) -> ::core::cmp::Ordering {
            ::core::cmp::Ord::cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for ClientStatePath {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state)
        }
    }
    impl ::core::fmt::Display for ClientStatePath {
        #[allow(unused_variables)]
        #[inline]
        fn fmt(
            &self,
            _derive_more_display_formatter: &mut ::core::fmt::Formatter,
        ) -> ::core::fmt::Result {
            match self {
                ClientStatePath(_0) => {
                    _derive_more_display_formatter
                        .write_fmt(format_args!("clients/{0}/clientState", _0))
                }
                _ => Ok(()),
            }
        }
    }
    impl ClientStatePath {
        pub fn new(client_id: &ClientId) -> ClientStatePath {
            ClientStatePath(client_id.clone())
        }
    }
    #[display(
        fmt = "clients/{client_id}/consensusStates/{revision_number}-{revision_height}"
    )]
    pub struct ClientConsensusStatePath {
        pub client_id: ClientId,
        pub revision_number: u64,
        pub revision_height: u64,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for ClientConsensusStatePath {
        #[inline]
        fn clone(&self) -> ClientConsensusStatePath {
            ClientConsensusStatePath {
                client_id: ::core::clone::Clone::clone(&self.client_id),
                revision_number: ::core::clone::Clone::clone(&self.revision_number),
                revision_height: ::core::clone::Clone::clone(&self.revision_height),
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ClientConsensusStatePath {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "ClientConsensusStatePath",
                "client_id",
                &self.client_id,
                "revision_number",
                &self.revision_number,
                "revision_height",
                &&self.revision_height,
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ClientConsensusStatePath {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ClientConsensusStatePath {
        #[inline]
        fn eq(&self, other: &ClientConsensusStatePath) -> bool {
            self.client_id == other.client_id
                && self.revision_number == other.revision_number
                && self.revision_height == other.revision_height
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for ClientConsensusStatePath {}
    #[automatically_derived]
    impl ::core::cmp::Eq for ClientConsensusStatePath {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<ClientId>;
            let _: ::core::cmp::AssertParamIsEq<u64>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ClientConsensusStatePath {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ClientConsensusStatePath,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            match ::core::cmp::PartialOrd::partial_cmp(
                &self.client_id,
                &other.client_id,
            ) {
                ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                    match ::core::cmp::PartialOrd::partial_cmp(
                        &self.revision_number,
                        &other.revision_number,
                    ) {
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                            ::core::cmp::PartialOrd::partial_cmp(
                                &self.revision_height,
                                &other.revision_height,
                            )
                        }
                        cmp => cmp,
                    }
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for ClientConsensusStatePath {
        #[inline]
        fn cmp(&self, other: &ClientConsensusStatePath) -> ::core::cmp::Ordering {
            match ::core::cmp::Ord::cmp(&self.client_id, &other.client_id) {
                ::core::cmp::Ordering::Equal => {
                    match ::core::cmp::Ord::cmp(
                        &self.revision_number,
                        &other.revision_number,
                    ) {
                        ::core::cmp::Ordering::Equal => {
                            ::core::cmp::Ord::cmp(
                                &self.revision_height,
                                &other.revision_height,
                            )
                        }
                        cmp => cmp,
                    }
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for ClientConsensusStatePath {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.client_id, state);
            ::core::hash::Hash::hash(&self.revision_number, state);
            ::core::hash::Hash::hash(&self.revision_height, state)
        }
    }
    impl ::core::fmt::Display for ClientConsensusStatePath {
        #[allow(unused_variables)]
        #[inline]
        fn fmt(
            &self,
            _derive_more_display_formatter: &mut ::core::fmt::Formatter,
        ) -> ::core::fmt::Result {
            match self {
                ClientConsensusStatePath {
                    client_id,
                    revision_number,
                    revision_height,
                } => {
                    _derive_more_display_formatter
                        .write_fmt(
                            format_args!(
                                "clients/{0}/consensusStates/{1}-{2}",
                                client_id,
                                revision_number,
                                revision_height,
                            ),
                        )
                }
                _ => Ok(()),
            }
        }
    }
    impl ClientConsensusStatePath {
        pub fn new(
            client_id: ClientId,
            revision_number: u64,
            revision_height: u64,
        ) -> ClientConsensusStatePath {
            ClientConsensusStatePath {
                client_id,
                revision_number,
                revision_height,
            }
        }
    }
    #[display(fmt = "clients/{_0}/connections")]
    pub struct ClientConnectionPath(pub ClientId);
    #[automatically_derived]
    impl ::core::clone::Clone for ClientConnectionPath {
        #[inline]
        fn clone(&self) -> ClientConnectionPath {
            ClientConnectionPath(::core::clone::Clone::clone(&self.0))
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ClientConnectionPath {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(
                f,
                "ClientConnectionPath",
                &&self.0,
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ClientConnectionPath {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ClientConnectionPath {
        #[inline]
        fn eq(&self, other: &ClientConnectionPath) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for ClientConnectionPath {}
    #[automatically_derived]
    impl ::core::cmp::Eq for ClientConnectionPath {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<ClientId>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ClientConnectionPath {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ClientConnectionPath,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for ClientConnectionPath {
        #[inline]
        fn cmp(&self, other: &ClientConnectionPath) -> ::core::cmp::Ordering {
            ::core::cmp::Ord::cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for ClientConnectionPath {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state)
        }
    }
    impl ::core::fmt::Display for ClientConnectionPath {
        #[allow(unused_variables)]
        #[inline]
        fn fmt(
            &self,
            _derive_more_display_formatter: &mut ::core::fmt::Formatter,
        ) -> ::core::fmt::Result {
            match self {
                ClientConnectionPath(_0) => {
                    _derive_more_display_formatter
                        .write_fmt(format_args!("clients/{0}/connections", _0))
                }
                _ => Ok(()),
            }
        }
    }
    impl ClientConnectionPath {
        pub fn new(client_id: &ClientId) -> ClientConnectionPath {
            ClientConnectionPath(client_id.clone())
        }
    }
    #[display(fmt = "connections/{_0}")]
    pub struct ConnectionPath(pub ConnectionId);
    #[automatically_derived]
    impl ::core::clone::Clone for ConnectionPath {
        #[inline]
        fn clone(&self) -> ConnectionPath {
            ConnectionPath(::core::clone::Clone::clone(&self.0))
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ConnectionPath {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(
                f,
                "ConnectionPath",
                &&self.0,
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ConnectionPath {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ConnectionPath {
        #[inline]
        fn eq(&self, other: &ConnectionPath) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for ConnectionPath {}
    #[automatically_derived]
    impl ::core::cmp::Eq for ConnectionPath {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<ConnectionId>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ConnectionPath {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ConnectionPath,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for ConnectionPath {
        #[inline]
        fn cmp(&self, other: &ConnectionPath) -> ::core::cmp::Ordering {
            ::core::cmp::Ord::cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for ConnectionPath {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state)
        }
    }
    impl ::core::fmt::Display for ConnectionPath {
        #[allow(unused_variables)]
        #[inline]
        fn fmt(
            &self,
            _derive_more_display_formatter: &mut ::core::fmt::Formatter,
        ) -> ::core::fmt::Result {
            match self {
                ConnectionPath(_0) => {
                    _derive_more_display_formatter
                        .write_fmt(format_args!("connections/{0}", _0))
                }
                _ => Ok(()),
            }
        }
    }
    impl ConnectionPath {
        pub fn new(connection_id: &ConnectionId) -> ConnectionPath {
            ConnectionPath(connection_id.clone())
        }
    }
    #[display(fmt = "ports/{_0}")]
    pub struct PortPath(pub PortId);
    #[automatically_derived]
    impl ::core::clone::Clone for PortPath {
        #[inline]
        fn clone(&self) -> PortPath {
            PortPath(::core::clone::Clone::clone(&self.0))
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for PortPath {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(f, "PortPath", &&self.0)
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for PortPath {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for PortPath {
        #[inline]
        fn eq(&self, other: &PortPath) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for PortPath {}
    #[automatically_derived]
    impl ::core::cmp::Eq for PortPath {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<PortId>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for PortPath {
        #[inline]
        fn partial_cmp(
            &self,
            other: &PortPath,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for PortPath {
        #[inline]
        fn cmp(&self, other: &PortPath) -> ::core::cmp::Ordering {
            ::core::cmp::Ord::cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for PortPath {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state)
        }
    }
    impl ::core::fmt::Display for PortPath {
        #[allow(unused_variables)]
        #[inline]
        fn fmt(
            &self,
            _derive_more_display_formatter: &mut ::core::fmt::Formatter,
        ) -> ::core::fmt::Result {
            match self {
                PortPath(_0) => {
                    _derive_more_display_formatter
                        .write_fmt(format_args!("ports/{0}", _0))
                }
                _ => Ok(()),
            }
        }
    }
    #[display(fmt = "channelEnds/ports/{_0}/channels/{_1}")]
    pub struct ChannelEndPath(pub PortId, pub ChannelId);
    #[automatically_derived]
    impl ::core::clone::Clone for ChannelEndPath {
        #[inline]
        fn clone(&self) -> ChannelEndPath {
            ChannelEndPath(
                ::core::clone::Clone::clone(&self.0),
                ::core::clone::Clone::clone(&self.1),
            )
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ChannelEndPath {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field2_finish(
                f,
                "ChannelEndPath",
                &self.0,
                &&self.1,
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ChannelEndPath {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ChannelEndPath {
        #[inline]
        fn eq(&self, other: &ChannelEndPath) -> bool {
            self.0 == other.0 && self.1 == other.1
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for ChannelEndPath {}
    #[automatically_derived]
    impl ::core::cmp::Eq for ChannelEndPath {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<PortId>;
            let _: ::core::cmp::AssertParamIsEq<ChannelId>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ChannelEndPath {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ChannelEndPath,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            match ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0) {
                ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                    ::core::cmp::PartialOrd::partial_cmp(&self.1, &other.1)
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for ChannelEndPath {
        #[inline]
        fn cmp(&self, other: &ChannelEndPath) -> ::core::cmp::Ordering {
            match ::core::cmp::Ord::cmp(&self.0, &other.0) {
                ::core::cmp::Ordering::Equal => ::core::cmp::Ord::cmp(&self.1, &other.1),
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for ChannelEndPath {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state);
            ::core::hash::Hash::hash(&self.1, state)
        }
    }
    impl ::core::fmt::Display for ChannelEndPath {
        #[allow(unused_variables)]
        #[inline]
        fn fmt(
            &self,
            _derive_more_display_formatter: &mut ::core::fmt::Formatter,
        ) -> ::core::fmt::Result {
            match self {
                ChannelEndPath(_0, _1) => {
                    _derive_more_display_formatter
                        .write_fmt(
                            format_args!("channelEnds/ports/{0}/channels/{1}", _0, _1),
                        )
                }
                _ => Ok(()),
            }
        }
    }
    impl ChannelEndPath {
        pub fn new(port_id: &PortId, channel_id: &ChannelId) -> ChannelEndPath {
            ChannelEndPath(port_id.clone(), channel_id.clone())
        }
    }
    #[display(fmt = "nextSequenceSend/ports/{_0}/channels/{_1}")]
    pub struct SeqSendPath(pub PortId, pub ChannelId);
    #[automatically_derived]
    impl ::core::clone::Clone for SeqSendPath {
        #[inline]
        fn clone(&self) -> SeqSendPath {
            SeqSendPath(
                ::core::clone::Clone::clone(&self.0),
                ::core::clone::Clone::clone(&self.1),
            )
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for SeqSendPath {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field2_finish(
                f,
                "SeqSendPath",
                &self.0,
                &&self.1,
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for SeqSendPath {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for SeqSendPath {
        #[inline]
        fn eq(&self, other: &SeqSendPath) -> bool {
            self.0 == other.0 && self.1 == other.1
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for SeqSendPath {}
    #[automatically_derived]
    impl ::core::cmp::Eq for SeqSendPath {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<PortId>;
            let _: ::core::cmp::AssertParamIsEq<ChannelId>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for SeqSendPath {
        #[inline]
        fn partial_cmp(
            &self,
            other: &SeqSendPath,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            match ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0) {
                ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                    ::core::cmp::PartialOrd::partial_cmp(&self.1, &other.1)
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for SeqSendPath {
        #[inline]
        fn cmp(&self, other: &SeqSendPath) -> ::core::cmp::Ordering {
            match ::core::cmp::Ord::cmp(&self.0, &other.0) {
                ::core::cmp::Ordering::Equal => ::core::cmp::Ord::cmp(&self.1, &other.1),
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for SeqSendPath {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state);
            ::core::hash::Hash::hash(&self.1, state)
        }
    }
    impl ::core::fmt::Display for SeqSendPath {
        #[allow(unused_variables)]
        #[inline]
        fn fmt(
            &self,
            _derive_more_display_formatter: &mut ::core::fmt::Formatter,
        ) -> ::core::fmt::Result {
            match self {
                SeqSendPath(_0, _1) => {
                    _derive_more_display_formatter
                        .write_fmt(
                            format_args!(
                                "nextSequenceSend/ports/{0}/channels/{1}",
                                _0,
                                _1,
                            ),
                        )
                }
                _ => Ok(()),
            }
        }
    }
    impl SeqSendPath {
        pub fn new(port_id: &PortId, channel_id: &ChannelId) -> SeqSendPath {
            SeqSendPath(port_id.clone(), channel_id.clone())
        }
    }
    #[display(fmt = "nextSequenceRecv/ports/{_0}/channels/{_1}")]
    pub struct SeqRecvPath(pub PortId, pub ChannelId);
    #[automatically_derived]
    impl ::core::clone::Clone for SeqRecvPath {
        #[inline]
        fn clone(&self) -> SeqRecvPath {
            SeqRecvPath(
                ::core::clone::Clone::clone(&self.0),
                ::core::clone::Clone::clone(&self.1),
            )
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for SeqRecvPath {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field2_finish(
                f,
                "SeqRecvPath",
                &self.0,
                &&self.1,
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for SeqRecvPath {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for SeqRecvPath {
        #[inline]
        fn eq(&self, other: &SeqRecvPath) -> bool {
            self.0 == other.0 && self.1 == other.1
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for SeqRecvPath {}
    #[automatically_derived]
    impl ::core::cmp::Eq for SeqRecvPath {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<PortId>;
            let _: ::core::cmp::AssertParamIsEq<ChannelId>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for SeqRecvPath {
        #[inline]
        fn partial_cmp(
            &self,
            other: &SeqRecvPath,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            match ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0) {
                ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                    ::core::cmp::PartialOrd::partial_cmp(&self.1, &other.1)
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for SeqRecvPath {
        #[inline]
        fn cmp(&self, other: &SeqRecvPath) -> ::core::cmp::Ordering {
            match ::core::cmp::Ord::cmp(&self.0, &other.0) {
                ::core::cmp::Ordering::Equal => ::core::cmp::Ord::cmp(&self.1, &other.1),
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for SeqRecvPath {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state);
            ::core::hash::Hash::hash(&self.1, state)
        }
    }
    impl ::core::fmt::Display for SeqRecvPath {
        #[allow(unused_variables)]
        #[inline]
        fn fmt(
            &self,
            _derive_more_display_formatter: &mut ::core::fmt::Formatter,
        ) -> ::core::fmt::Result {
            match self {
                SeqRecvPath(_0, _1) => {
                    _derive_more_display_formatter
                        .write_fmt(
                            format_args!(
                                "nextSequenceRecv/ports/{0}/channels/{1}",
                                _0,
                                _1,
                            ),
                        )
                }
                _ => Ok(()),
            }
        }
    }
    impl SeqRecvPath {
        pub fn new(port_id: &PortId, channel_id: &ChannelId) -> SeqRecvPath {
            SeqRecvPath(port_id.clone(), channel_id.clone())
        }
    }
    #[display(fmt = "nextSequenceAck/ports/{_0}/channels/{_1}")]
    pub struct SeqAckPath(pub PortId, pub ChannelId);
    #[automatically_derived]
    impl ::core::clone::Clone for SeqAckPath {
        #[inline]
        fn clone(&self) -> SeqAckPath {
            SeqAckPath(
                ::core::clone::Clone::clone(&self.0),
                ::core::clone::Clone::clone(&self.1),
            )
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for SeqAckPath {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field2_finish(
                f,
                "SeqAckPath",
                &self.0,
                &&self.1,
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for SeqAckPath {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for SeqAckPath {
        #[inline]
        fn eq(&self, other: &SeqAckPath) -> bool {
            self.0 == other.0 && self.1 == other.1
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for SeqAckPath {}
    #[automatically_derived]
    impl ::core::cmp::Eq for SeqAckPath {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<PortId>;
            let _: ::core::cmp::AssertParamIsEq<ChannelId>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for SeqAckPath {
        #[inline]
        fn partial_cmp(
            &self,
            other: &SeqAckPath,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            match ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0) {
                ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                    ::core::cmp::PartialOrd::partial_cmp(&self.1, &other.1)
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for SeqAckPath {
        #[inline]
        fn cmp(&self, other: &SeqAckPath) -> ::core::cmp::Ordering {
            match ::core::cmp::Ord::cmp(&self.0, &other.0) {
                ::core::cmp::Ordering::Equal => ::core::cmp::Ord::cmp(&self.1, &other.1),
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for SeqAckPath {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state);
            ::core::hash::Hash::hash(&self.1, state)
        }
    }
    impl ::core::fmt::Display for SeqAckPath {
        #[allow(unused_variables)]
        #[inline]
        fn fmt(
            &self,
            _derive_more_display_formatter: &mut ::core::fmt::Formatter,
        ) -> ::core::fmt::Result {
            match self {
                SeqAckPath(_0, _1) => {
                    _derive_more_display_formatter
                        .write_fmt(
                            format_args!(
                                "nextSequenceAck/ports/{0}/channels/{1}",
                                _0,
                                _1,
                            ),
                        )
                }
                _ => Ok(()),
            }
        }
    }
    impl SeqAckPath {
        pub fn new(port_id: &PortId, channel_id: &ChannelId) -> SeqAckPath {
            SeqAckPath(port_id.clone(), channel_id.clone())
        }
    }
    #[display(
        fmt = "commitments/ports/{port_id}/channels/{channel_id}/sequences/{sequence}"
    )]
    pub struct CommitmentPath {
        pub port_id: PortId,
        pub channel_id: ChannelId,
        pub sequence: Sequence,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for CommitmentPath {
        #[inline]
        fn clone(&self) -> CommitmentPath {
            CommitmentPath {
                port_id: ::core::clone::Clone::clone(&self.port_id),
                channel_id: ::core::clone::Clone::clone(&self.channel_id),
                sequence: ::core::clone::Clone::clone(&self.sequence),
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for CommitmentPath {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "CommitmentPath",
                "port_id",
                &self.port_id,
                "channel_id",
                &self.channel_id,
                "sequence",
                &&self.sequence,
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for CommitmentPath {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for CommitmentPath {
        #[inline]
        fn eq(&self, other: &CommitmentPath) -> bool {
            self.port_id == other.port_id && self.channel_id == other.channel_id
                && self.sequence == other.sequence
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for CommitmentPath {}
    #[automatically_derived]
    impl ::core::cmp::Eq for CommitmentPath {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<PortId>;
            let _: ::core::cmp::AssertParamIsEq<ChannelId>;
            let _: ::core::cmp::AssertParamIsEq<Sequence>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for CommitmentPath {
        #[inline]
        fn partial_cmp(
            &self,
            other: &CommitmentPath,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            match ::core::cmp::PartialOrd::partial_cmp(&self.port_id, &other.port_id) {
                ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                    match ::core::cmp::PartialOrd::partial_cmp(
                        &self.channel_id,
                        &other.channel_id,
                    ) {
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                            ::core::cmp::PartialOrd::partial_cmp(
                                &self.sequence,
                                &other.sequence,
                            )
                        }
                        cmp => cmp,
                    }
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for CommitmentPath {
        #[inline]
        fn cmp(&self, other: &CommitmentPath) -> ::core::cmp::Ordering {
            match ::core::cmp::Ord::cmp(&self.port_id, &other.port_id) {
                ::core::cmp::Ordering::Equal => {
                    match ::core::cmp::Ord::cmp(&self.channel_id, &other.channel_id) {
                        ::core::cmp::Ordering::Equal => {
                            ::core::cmp::Ord::cmp(&self.sequence, &other.sequence)
                        }
                        cmp => cmp,
                    }
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for CommitmentPath {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.port_id, state);
            ::core::hash::Hash::hash(&self.channel_id, state);
            ::core::hash::Hash::hash(&self.sequence, state)
        }
    }
    impl ::core::fmt::Display for CommitmentPath {
        #[allow(unused_variables)]
        #[inline]
        fn fmt(
            &self,
            _derive_more_display_formatter: &mut ::core::fmt::Formatter,
        ) -> ::core::fmt::Result {
            match self {
                CommitmentPath { port_id, channel_id, sequence } => {
                    _derive_more_display_formatter
                        .write_fmt(
                            format_args!(
                                "commitments/ports/{0}/channels/{1}/sequences/{2}",
                                port_id,
                                channel_id,
                                sequence,
                            ),
                        )
                }
                _ => Ok(()),
            }
        }
    }
    impl CommitmentPath {
        pub fn new(
            port_id: &PortId,
            channel_id: &ChannelId,
            sequence: Sequence,
        ) -> CommitmentPath {
            CommitmentPath {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
                sequence,
            }
        }
    }
    #[display(fmt = "acks/ports/{port_id}/channels/{channel_id}/sequences/{sequence}")]
    pub struct AckPath {
        pub port_id: PortId,
        pub channel_id: ChannelId,
        pub sequence: Sequence,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for AckPath {
        #[inline]
        fn clone(&self) -> AckPath {
            AckPath {
                port_id: ::core::clone::Clone::clone(&self.port_id),
                channel_id: ::core::clone::Clone::clone(&self.channel_id),
                sequence: ::core::clone::Clone::clone(&self.sequence),
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for AckPath {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "AckPath",
                "port_id",
                &self.port_id,
                "channel_id",
                &self.channel_id,
                "sequence",
                &&self.sequence,
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for AckPath {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for AckPath {
        #[inline]
        fn eq(&self, other: &AckPath) -> bool {
            self.port_id == other.port_id && self.channel_id == other.channel_id
                && self.sequence == other.sequence
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for AckPath {}
    #[automatically_derived]
    impl ::core::cmp::Eq for AckPath {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<PortId>;
            let _: ::core::cmp::AssertParamIsEq<ChannelId>;
            let _: ::core::cmp::AssertParamIsEq<Sequence>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for AckPath {
        #[inline]
        fn partial_cmp(
            &self,
            other: &AckPath,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            match ::core::cmp::PartialOrd::partial_cmp(&self.port_id, &other.port_id) {
                ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                    match ::core::cmp::PartialOrd::partial_cmp(
                        &self.channel_id,
                        &other.channel_id,
                    ) {
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                            ::core::cmp::PartialOrd::partial_cmp(
                                &self.sequence,
                                &other.sequence,
                            )
                        }
                        cmp => cmp,
                    }
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for AckPath {
        #[inline]
        fn cmp(&self, other: &AckPath) -> ::core::cmp::Ordering {
            match ::core::cmp::Ord::cmp(&self.port_id, &other.port_id) {
                ::core::cmp::Ordering::Equal => {
                    match ::core::cmp::Ord::cmp(&self.channel_id, &other.channel_id) {
                        ::core::cmp::Ordering::Equal => {
                            ::core::cmp::Ord::cmp(&self.sequence, &other.sequence)
                        }
                        cmp => cmp,
                    }
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for AckPath {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.port_id, state);
            ::core::hash::Hash::hash(&self.channel_id, state);
            ::core::hash::Hash::hash(&self.sequence, state)
        }
    }
    impl ::core::fmt::Display for AckPath {
        #[allow(unused_variables)]
        #[inline]
        fn fmt(
            &self,
            _derive_more_display_formatter: &mut ::core::fmt::Formatter,
        ) -> ::core::fmt::Result {
            match self {
                AckPath { port_id, channel_id, sequence } => {
                    _derive_more_display_formatter
                        .write_fmt(
                            format_args!(
                                "acks/ports/{0}/channels/{1}/sequences/{2}",
                                port_id,
                                channel_id,
                                sequence,
                            ),
                        )
                }
                _ => Ok(()),
            }
        }
    }
    impl AckPath {
        pub fn new(
            port_id: &PortId,
            channel_id: &ChannelId,
            sequence: Sequence,
        ) -> AckPath {
            AckPath {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
                sequence,
            }
        }
    }
    #[display(
        fmt = "receipts/ports/{port_id}/channels/{channel_id}/sequences/{sequence}"
    )]
    pub struct ReceiptPath {
        pub port_id: PortId,
        pub channel_id: ChannelId,
        pub sequence: Sequence,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for ReceiptPath {
        #[inline]
        fn clone(&self) -> ReceiptPath {
            ReceiptPath {
                port_id: ::core::clone::Clone::clone(&self.port_id),
                channel_id: ::core::clone::Clone::clone(&self.channel_id),
                sequence: ::core::clone::Clone::clone(&self.sequence),
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ReceiptPath {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "ReceiptPath",
                "port_id",
                &self.port_id,
                "channel_id",
                &self.channel_id,
                "sequence",
                &&self.sequence,
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ReceiptPath {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ReceiptPath {
        #[inline]
        fn eq(&self, other: &ReceiptPath) -> bool {
            self.port_id == other.port_id && self.channel_id == other.channel_id
                && self.sequence == other.sequence
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for ReceiptPath {}
    #[automatically_derived]
    impl ::core::cmp::Eq for ReceiptPath {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<PortId>;
            let _: ::core::cmp::AssertParamIsEq<ChannelId>;
            let _: ::core::cmp::AssertParamIsEq<Sequence>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ReceiptPath {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ReceiptPath,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            match ::core::cmp::PartialOrd::partial_cmp(&self.port_id, &other.port_id) {
                ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                    match ::core::cmp::PartialOrd::partial_cmp(
                        &self.channel_id,
                        &other.channel_id,
                    ) {
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                            ::core::cmp::PartialOrd::partial_cmp(
                                &self.sequence,
                                &other.sequence,
                            )
                        }
                        cmp => cmp,
                    }
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for ReceiptPath {
        #[inline]
        fn cmp(&self, other: &ReceiptPath) -> ::core::cmp::Ordering {
            match ::core::cmp::Ord::cmp(&self.port_id, &other.port_id) {
                ::core::cmp::Ordering::Equal => {
                    match ::core::cmp::Ord::cmp(&self.channel_id, &other.channel_id) {
                        ::core::cmp::Ordering::Equal => {
                            ::core::cmp::Ord::cmp(&self.sequence, &other.sequence)
                        }
                        cmp => cmp,
                    }
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for ReceiptPath {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.port_id, state);
            ::core::hash::Hash::hash(&self.channel_id, state);
            ::core::hash::Hash::hash(&self.sequence, state)
        }
    }
    impl ::core::fmt::Display for ReceiptPath {
        #[allow(unused_variables)]
        #[inline]
        fn fmt(
            &self,
            _derive_more_display_formatter: &mut ::core::fmt::Formatter,
        ) -> ::core::fmt::Result {
            match self {
                ReceiptPath { port_id, channel_id, sequence } => {
                    _derive_more_display_formatter
                        .write_fmt(
                            format_args!(
                                "receipts/ports/{0}/channels/{1}/sequences/{2}",
                                port_id,
                                channel_id,
                                sequence,
                            ),
                        )
                }
                _ => Ok(()),
            }
        }
    }
    impl ReceiptPath {
        pub fn new(
            port_id: &PortId,
            channel_id: &ChannelId,
            sequence: Sequence,
        ) -> ReceiptPath {
            ReceiptPath {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
                sequence,
            }
        }
    }
    /// Paths that are specific for client upgrades.
    pub enum UpgradeClientPath {
        #[display(fmt = "{UPGRADED_IBC_STATE}/{_0}/{UPGRADED_CLIENT_STATE}")]
        UpgradedClientState(u64),
        #[display(fmt = "{UPGRADED_IBC_STATE}/{_0}/{UPGRADED_CLIENT_CONSENSUS_STATE}")]
        UpgradedClientConsensusState(u64),
    }
    #[automatically_derived]
    impl ::core::clone::Clone for UpgradeClientPath {
        #[inline]
        fn clone(&self) -> UpgradeClientPath {
            match self {
                UpgradeClientPath::UpgradedClientState(__self_0) => {
                    UpgradeClientPath::UpgradedClientState(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                UpgradeClientPath::UpgradedClientConsensusState(__self_0) => {
                    UpgradeClientPath::UpgradedClientConsensusState(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UpgradeClientPath {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                UpgradeClientPath::UpgradedClientState(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "UpgradedClientState",
                        &__self_0,
                    )
                }
                UpgradeClientPath::UpgradedClientConsensusState(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "UpgradedClientConsensusState",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for UpgradeClientPath {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for UpgradeClientPath {
        #[inline]
        fn eq(&self, other: &UpgradeClientPath) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
                && match (self, other) {
                    (
                        UpgradeClientPath::UpgradedClientState(__self_0),
                        UpgradeClientPath::UpgradedClientState(__arg1_0),
                    ) => *__self_0 == *__arg1_0,
                    (
                        UpgradeClientPath::UpgradedClientConsensusState(__self_0),
                        UpgradeClientPath::UpgradedClientConsensusState(__arg1_0),
                    ) => *__self_0 == *__arg1_0,
                    _ => unsafe { ::core::intrinsics::unreachable() }
                }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for UpgradeClientPath {}
    #[automatically_derived]
    impl ::core::cmp::Eq for UpgradeClientPath {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<u64>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for UpgradeClientPath {
        #[inline]
        fn partial_cmp(
            &self,
            other: &UpgradeClientPath,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            match (self, other) {
                (
                    UpgradeClientPath::UpgradedClientState(__self_0),
                    UpgradeClientPath::UpgradedClientState(__arg1_0),
                ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
                (
                    UpgradeClientPath::UpgradedClientConsensusState(__self_0),
                    UpgradeClientPath::UpgradedClientConsensusState(__arg1_0),
                ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
                _ => ::core::cmp::PartialOrd::partial_cmp(&__self_tag, &__arg1_tag),
            }
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for UpgradeClientPath {
        #[inline]
        fn cmp(&self, other: &UpgradeClientPath) -> ::core::cmp::Ordering {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            match ::core::cmp::Ord::cmp(&__self_tag, &__arg1_tag) {
                ::core::cmp::Ordering::Equal => {
                    match (self, other) {
                        (
                            UpgradeClientPath::UpgradedClientState(__self_0),
                            UpgradeClientPath::UpgradedClientState(__arg1_0),
                        ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                        (
                            UpgradeClientPath::UpgradedClientConsensusState(__self_0),
                            UpgradeClientPath::UpgradedClientConsensusState(__arg1_0),
                        ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                        _ => unsafe { ::core::intrinsics::unreachable() }
                    }
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for UpgradeClientPath {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_tag, state);
            match self {
                UpgradeClientPath::UpgradedClientState(__self_0) => {
                    ::core::hash::Hash::hash(__self_0, state)
                }
                UpgradeClientPath::UpgradedClientConsensusState(__self_0) => {
                    ::core::hash::Hash::hash(__self_0, state)
                }
            }
        }
    }
    impl ::core::fmt::Display for UpgradeClientPath {
        #[allow(unused_variables)]
        #[inline]
        fn fmt(
            &self,
            _derive_more_display_formatter: &mut ::core::fmt::Formatter,
        ) -> ::core::fmt::Result {
            match self {
                UpgradeClientPath::UpgradedClientState(_0) => {
                    _derive_more_display_formatter
                        .write_fmt(
                            format_args!(
                                "{0}/{1}/{2}",
                                UPGRADED_IBC_STATE,
                                _0,
                                UPGRADED_CLIENT_STATE,
                            ),
                        )
                }
                UpgradeClientPath::UpgradedClientConsensusState(_0) => {
                    _derive_more_display_formatter
                        .write_fmt(
                            format_args!(
                                "{0}/{1}/{2}",
                                UPGRADED_IBC_STATE,
                                _0,
                                UPGRADED_CLIENT_CONSENSUS_STATE,
                            ),
                        )
                }
                _ => Ok(()),
            }
        }
    }
    /// Sub-paths which are not part of the specification, but are still
    /// useful to represent for parsing purposes.
    enum SubPath {
        Channels(ChannelId),
        Sequences(Sequence),
    }
    #[automatically_derived]
    impl ::core::clone::Clone for SubPath {
        #[inline]
        fn clone(&self) -> SubPath {
            match self {
                SubPath::Channels(__self_0) => {
                    SubPath::Channels(::core::clone::Clone::clone(__self_0))
                }
                SubPath::Sequences(__self_0) => {
                    SubPath::Sequences(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for SubPath {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                SubPath::Channels(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Channels",
                        &__self_0,
                    )
                }
                SubPath::Sequences(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Sequences",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for SubPath {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for SubPath {
        #[inline]
        fn eq(&self, other: &SubPath) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
                && match (self, other) {
                    (SubPath::Channels(__self_0), SubPath::Channels(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (SubPath::Sequences(__self_0), SubPath::Sequences(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    _ => unsafe { ::core::intrinsics::unreachable() }
                }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for SubPath {}
    #[automatically_derived]
    impl ::core::cmp::Eq for SubPath {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<ChannelId>;
            let _: ::core::cmp::AssertParamIsEq<Sequence>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for SubPath {
        #[inline]
        fn partial_cmp(
            &self,
            other: &SubPath,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            match (self, other) {
                (SubPath::Channels(__self_0), SubPath::Channels(__arg1_0)) => {
                    ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0)
                }
                (SubPath::Sequences(__self_0), SubPath::Sequences(__arg1_0)) => {
                    ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0)
                }
                _ => ::core::cmp::PartialOrd::partial_cmp(&__self_tag, &__arg1_tag),
            }
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for SubPath {
        #[inline]
        fn cmp(&self, other: &SubPath) -> ::core::cmp::Ordering {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            match ::core::cmp::Ord::cmp(&__self_tag, &__arg1_tag) {
                ::core::cmp::Ordering::Equal => {
                    match (self, other) {
                        (SubPath::Channels(__self_0), SubPath::Channels(__arg1_0)) => {
                            ::core::cmp::Ord::cmp(__self_0, __arg1_0)
                        }
                        (SubPath::Sequences(__self_0), SubPath::Sequences(__arg1_0)) => {
                            ::core::cmp::Ord::cmp(__self_0, __arg1_0)
                        }
                        _ => unsafe { ::core::intrinsics::unreachable() }
                    }
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for SubPath {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_tag, state);
            match self {
                SubPath::Channels(__self_0) => ::core::hash::Hash::hash(__self_0, state),
                SubPath::Sequences(__self_0) => ::core::hash::Hash::hash(__self_0, state),
            }
        }
    }
    impl Path {
        /// Indication if the path is provable.
        pub fn is_provable(&self) -> bool {
            !match &self {
                Path::ClientConnection(_) | Path::Ports(_) => true,
                _ => false,
            }
        }
        /// into_bytes implementation
        pub fn into_bytes(self) -> Vec<u8> {
            self.to_string().into_bytes()
        }
    }
    pub enum PathError {
        /// `{path}` could not be parsed into a Path
        ParseFailure { path: String },
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for PathError {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                PathError::ParseFailure { path: __self_0 } => {
                    ::core::fmt::Formatter::debug_struct_field1_finish(
                        f,
                        "ParseFailure",
                        "path",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _DERIVE_Display_FOR_PathError: () = {
        trait DisplayToDisplayDoc {
            fn __displaydoc_display(&self) -> Self;
        }
        impl<T: core::fmt::Display> DisplayToDisplayDoc for &T {
            fn __displaydoc_display(&self) -> Self {
                self
            }
        }
        extern crate std;
        trait PathToDisplayDoc {
            fn __displaydoc_display(&self) -> std::path::Display<'_>;
        }
        impl PathToDisplayDoc for std::path::Path {
            fn __displaydoc_display(&self) -> std::path::Display<'_> {
                self.display()
            }
        }
        impl PathToDisplayDoc for std::path::PathBuf {
            fn __displaydoc_display(&self) -> std::path::Display<'_> {
                self.display()
            }
        }
        impl core::fmt::Display for PathError {
            fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                #[allow(unused_variables)]
                match self {
                    Self::ParseFailure { path } => {
                        formatter
                            .write_fmt(
                                format_args!(
                                    "`{0}` could not be parsed into a Path",
                                    path.__displaydoc_display(),
                                ),
                            )
                    }
                }
            }
        }
    };
    #[cfg(feature = "std")]
    impl std::error::Error for PathError {}
    /// The FromStr trait allows paths encoded as strings to be parsed into Paths.
    impl FromStr for Path {
        type Err = PathError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let components: Vec<&str> = s.split('/').collect();
            parse_client_paths(&components)
                .or_else(|| parse_connections(&components))
                .or_else(|| parse_ports(&components))
                .or_else(|| parse_channel_ends(&components))
                .or_else(|| parse_seqs(&components))
                .or_else(|| parse_commitments(&components))
                .or_else(|| parse_acks(&components))
                .or_else(|| parse_receipts(&components))
                .or_else(|| parse_upgrades(&components))
                .ok_or(PathError::ParseFailure {
                    path: s.to_string(),
                })
        }
    }
    fn parse_client_paths(components: &[&str]) -> Option<Path> {
        let first = match components.first() {
            Some(f) => *f,
            None => return None,
        };
        if first != "clients" {
            return None;
        }
        let client_id = match ClientId::from_str(components[1]) {
            Ok(s) => s,
            Err(_) => return None,
        };
        if components.len() == 3 {
            match components[2] {
                "clientState" => Some(ClientStatePath(client_id).into()),
                "connections" => Some(ClientConnectionPath(client_id).into()),
                _ => None,
            }
        } else if components.len() == 4 {
            if "consensusStates" != components[2] {
                return None;
            }
            let epoch_height = match components.last() {
                Some(eh) => *eh,
                None => return None,
            };
            let epoch_height: Vec<&str> = epoch_height.split('-').collect();
            if epoch_height.len() != 2 {
                return None;
            }
            let revision_number = epoch_height[0];
            let revision_height = epoch_height[1];
            let revision_number = match revision_number.parse::<u64>() {
                Ok(ep) => ep,
                Err(_) => return None,
            };
            let revision_height = match revision_height.parse::<u64>() {
                Ok(h) => h,
                Err(_) => return None,
            };
            Some(
                ClientConsensusStatePath {
                    client_id,
                    revision_number,
                    revision_height,
                }
                    .into(),
            )
        } else {
            None
        }
    }
    fn parse_connections(components: &[&str]) -> Option<Path> {
        if components.len() != 2 {
            return None;
        }
        let first = match components.first() {
            Some(f) => *f,
            None => return None,
        };
        if first != "connections" {
            return None;
        }
        let connection_id = match components.last() {
            Some(c) => *c,
            None => return None,
        };
        let connection_id = match ConnectionId::from_str(connection_id) {
            Ok(c) => c,
            Err(_) => return None,
        };
        Some(ConnectionPath(connection_id).into())
    }
    fn parse_ports(components: &[&str]) -> Option<Path> {
        if components.len() != 2 {
            return None;
        }
        let first = match components.first() {
            Some(f) => *f,
            None => return None,
        };
        if first != "ports" {
            return None;
        }
        let port_id = match components.last() {
            Some(p) => *p,
            None => return None,
        };
        let port_id = match PortId::from_str(port_id) {
            Ok(p) => p,
            Err(_) => return None,
        };
        Some(PortPath(port_id).into())
    }
    fn parse_channels(components: &[&str]) -> Option<SubPath> {
        if components.len() != 2 {
            return None;
        }
        let first = match components.first() {
            Some(f) => *f,
            None => return None,
        };
        if first != "channels" {
            return None;
        }
        let channel_id = match components.last() {
            Some(c) => *c,
            None => return None,
        };
        let channel_id = match ChannelId::from_str(channel_id) {
            Ok(c) => c,
            Err(_) => return None,
        };
        Some(SubPath::Channels(channel_id))
    }
    fn parse_sequences(components: &[&str]) -> Option<SubPath> {
        if components.len() != 2 {
            return None;
        }
        let first = match components.first() {
            Some(f) => *f,
            None => return None,
        };
        if first != "sequences" {
            return None;
        }
        let sequence_number = match components.last() {
            Some(s) => *s,
            None => return None,
        };
        match Sequence::from_str(sequence_number) {
            Ok(seq) => Some(SubPath::Sequences(seq)),
            Err(_) => None,
        }
    }
    fn parse_channel_ends(components: &[&str]) -> Option<Path> {
        if components.len() != 5 {
            return None;
        }
        let first = match components.first() {
            Some(f) => *f,
            None => return None,
        };
        if first != "channelEnds" {
            return None;
        }
        let port = parse_ports(&components[1..=2]);
        let channel = parse_channels(&components[3..=4]);
        let port_id = if let Some(Path::Ports(PortPath(port_id))) = port {
            port_id
        } else {
            return None;
        };
        let channel_id = if let Some(SubPath::Channels(channel_id)) = channel {
            channel_id
        } else {
            return None;
        };
        Some(ChannelEndPath(port_id, channel_id).into())
    }
    fn parse_seqs(components: &[&str]) -> Option<Path> {
        if components.len() != 5 {
            return None;
        }
        let first = match components.first() {
            Some(f) => *f,
            None => return None,
        };
        let port = parse_ports(&components[1..=2]);
        let channel = parse_channels(&components[3..=4]);
        let port_id = if let Some(Path::Ports(PortPath(port_id))) = port {
            port_id
        } else {
            return None;
        };
        let channel_id = if let Some(SubPath::Channels(channel_id)) = channel {
            channel_id
        } else {
            return None;
        };
        match first {
            "nextSequenceSend" => Some(SeqSendPath(port_id, channel_id).into()),
            "nextSequenceRecv" => Some(SeqRecvPath(port_id, channel_id).into()),
            "nextSequenceAck" => Some(SeqAckPath(port_id, channel_id).into()),
            _ => None,
        }
    }
    fn parse_commitments(components: &[&str]) -> Option<Path> {
        if components.len() != 7 {
            return None;
        }
        let first = match components.first() {
            Some(f) => *f,
            None => return None,
        };
        if first != "commitments" {
            return None;
        }
        let port = parse_ports(&components[1..=2]);
        let channel = parse_channels(&components[3..=4]);
        let sequence = parse_sequences(&components[5..]);
        let port_id = if let Some(Path::Ports(PortPath(port_id))) = port {
            port_id
        } else {
            return None;
        };
        let channel_id = if let Some(SubPath::Channels(channel_id)) = channel {
            channel_id
        } else {
            return None;
        };
        let sequence = if let Some(SubPath::Sequences(seq)) = sequence {
            seq
        } else {
            return None;
        };
        Some(
            CommitmentPath {
                port_id,
                channel_id,
                sequence,
            }
                .into(),
        )
    }
    fn parse_acks(components: &[&str]) -> Option<Path> {
        if components.len() != 7 {
            return None;
        }
        let first = match components.first() {
            Some(f) => *f,
            None => return None,
        };
        if first != "acks" {
            return None;
        }
        let port = parse_ports(&components[1..=2]);
        let channel = parse_channels(&components[3..=4]);
        let sequence = parse_sequences(&components[5..]);
        let port_id = if let Some(Path::Ports(PortPath(port_id))) = port {
            port_id
        } else {
            return None;
        };
        let channel_id = if let Some(SubPath::Channels(channel_id)) = channel {
            channel_id
        } else {
            return None;
        };
        let sequence = if let Some(SubPath::Sequences(seq)) = sequence {
            seq
        } else {
            return None;
        };
        Some(
            AckPath {
                port_id,
                channel_id,
                sequence,
            }
                .into(),
        )
    }
    fn parse_receipts(components: &[&str]) -> Option<Path> {
        if components.len() != 7 {
            return None;
        }
        let first = match components.first() {
            Some(f) => *f,
            None => return None,
        };
        if first != "receipts" {
            return None;
        }
        let port = parse_ports(&components[1..=2]);
        let channel = parse_channels(&components[3..=4]);
        let sequence = parse_sequences(&components[5..]);
        let port_id = if let Some(Path::Ports(PortPath(port_id))) = port {
            port_id
        } else {
            return None;
        };
        let channel_id = if let Some(SubPath::Channels(channel_id)) = channel {
            channel_id
        } else {
            return None;
        };
        let sequence = if let Some(SubPath::Sequences(seq)) = sequence {
            seq
        } else {
            return None;
        };
        Some(
            ReceiptPath {
                port_id,
                channel_id,
                sequence,
            }
                .into(),
        )
    }
    fn parse_upgrades(components: &[&str]) -> Option<Path> {
        if components.len() != 3 {
            return None;
        }
        let first = match components.first() {
            Some(f) => *f,
            None => return None,
        };
        if first != UPGRADED_IBC_STATE {
            return None;
        }
        let last = match components.last() {
            Some(l) => *l,
            None => return None,
        };
        let height = match components[1].parse::<u64>() {
            Ok(h) => h,
            Err(_) => return None,
        };
        match last {
            UPGRADED_CLIENT_STATE => {
                Some(UpgradeClientPath::UpgradedClientState(height).into())
            }
            UPGRADED_CLIENT_CONSENSUS_STATE => {
                Some(UpgradeClientPath::UpgradedClientConsensusState(height).into())
            }
            _ => None,
        }
    }
}
pub(crate) mod validate {
    use ibc_primitives::prelude::*;
    use crate::error::IdentifierError as Error;
    const VALID_SPECIAL_CHARS: &str = "._+-#[]<>";
    /// Checks if the identifier only contains valid characters as specified in the
    /// [`ICS-24`](https://github.com/cosmos/ibc/tree/main/spec/core/ics-024-host-requirements#paths-identifiers-separators)]
    /// spec.
    pub fn validate_identifier_chars(id: &str) -> Result<(), Error> {
        if !id.chars().all(|c| c.is_alphanumeric() || VALID_SPECIAL_CHARS.contains(c)) {
            return Err(Error::InvalidCharacter {
                id: id.into(),
            });
        }
        Ok(())
    }
    /// Checks if the identifier forms a valid identifier with the given min/max length as specified in the
    /// [`ICS-24`](https://github.com/cosmos/ibc/tree/main/spec/core/ics-024-host-requirements#paths-identifiers-separators)]
    /// spec.
    pub fn validate_identifier_length(
        id: &str,
        min: u64,
        max: u64,
    ) -> Result<(), Error> {
        let min = min.max(1);
        let length = id.len() as u64;
        if (min..=max).contains(&length) {
            Ok(())
        } else {
            Err(Error::InvalidLength {
                id: id.into(),
                min,
                max,
            })
        }
    }
    /// Checks if a prefix forms a valid identifier with the given min/max identifier's length.
    /// The prefix must be between `min_id_length - 2`, considering `u64::MIN` (1 char) and "-"
    /// and `max_id_length - 21` characters, considering `u64::MAX` (20 chars) and "-".
    pub fn validate_prefix_length(
        prefix: &str,
        min_id_length: u64,
        max_id_length: u64,
    ) -> Result<(), Error> {
        let min = min_id_length.saturating_sub(2);
        let max = max_id_length.saturating_sub(21);
        validate_identifier_length(prefix, min, max)
    }
    /// Default validator function for the Client types.
    pub fn validate_client_type(id: &str) -> Result<(), Error> {
        validate_identifier_chars(id)?;
        validate_prefix_length(id, 9, 64)
    }
    /// Default validator function for Client identifiers.
    ///
    /// A valid client identifier must be between 9-64 characters as specified in
    /// the ICS-24 spec.
    pub fn validate_client_identifier(id: &str) -> Result<(), Error> {
        validate_identifier_chars(id)?;
        validate_identifier_length(id, 9, 64)
    }
    /// Default validator function for Connection identifiers.
    ///
    /// A valid connection identifier must be between 10-64 characters as specified
    /// in the ICS-24 spec.
    pub fn validate_connection_identifier(id: &str) -> Result<(), Error> {
        validate_identifier_chars(id)?;
        validate_identifier_length(id, 10, 64)
    }
    /// Default validator function for Port identifiers.
    ///
    /// A valid port identifier must be between 2-128 characters as specified in the
    /// ICS-24 spec.
    pub fn validate_port_identifier(id: &str) -> Result<(), Error> {
        validate_identifier_chars(id)?;
        validate_identifier_length(id, 2, 128)
    }
    /// Default validator function for Channel identifiers.
    ///
    /// A valid channel identifier must be between 8-64 characters as specified in
    /// the ICS-24 spec.
    pub fn validate_channel_identifier(id: &str) -> Result<(), Error> {
        validate_identifier_chars(id)?;
        validate_identifier_length(id, 8, 64)
    }
}
