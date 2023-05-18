//! Defines the upgrade error type

use alloc::string::String;
use displaydoc::Display;

#[derive(Debug, Display)]
pub enum UpgradeError {
    /// invalid upgrade proposal: `{reason}`
    InvalidUpgradeProposal { reason: String },
    /// invalid upgrade plan: `{reason}`
    InvalidUpgradePlan { reason: String },
    /// other: `{reason}`
    Other { reason: String },
}

#[cfg(feature = "std")]
impl std::error::Error for UpgradeError {}
