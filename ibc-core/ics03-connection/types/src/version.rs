//! Defines connection versioning type and functions

use core::fmt::Display;

use ibc_primitives::prelude::*;
use ibc_primitives::utils::PrettySlice;
use ibc_proto::ibc::core::connection::v1::Version as RawVersion;
use ibc_proto::Protobuf;

use crate::error::ConnectionError;

/// Stores the identifier and the features supported by a version
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
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Version {
    /// unique version identifier
    identifier: String,
    /// list of features compatible with the specified identifier
    features: Vec<String>,
}

impl Version {
    /// Checks whether the version has a matching version identifier and its
    /// feature set is a subset of the supported features
    pub fn verify_is_supported(
        &self,
        supported_versions: &[Version],
    ) -> Result<(), ConnectionError> {
        let maybe_supported_version = find_supported_version(self, supported_versions)?;

        if self.features.is_empty() {
            return Err(ConnectionError::EmptyFeatures);
        }

        for feature in self.features.iter() {
            maybe_supported_version.verify_feature_supported(feature.to_string())?;
        }
        Ok(())
    }

    /// Checks whether the given feature is supported in this version
    pub fn verify_feature_supported(&self, feature: String) -> Result<(), ConnectionError> {
        if !self.features.contains(&feature) {
            return Err(ConnectionError::FeatureNotSupported { feature });
        }
        Ok(())
    }

    /// Returns the lists of supported versions
    pub fn compatibles() -> Vec<Self> {
        vec![Self {
            identifier: "1".to_string(),
            features: vec!["ORDER_ORDERED".to_string(), "ORDER_UNORDERED".to_string()],
        }]
    }
}

impl Protobuf<RawVersion> for Version {}

impl TryFrom<RawVersion> for Version {
    type Error = ConnectionError;
    fn try_from(value: RawVersion) -> Result<Self, Self::Error> {
        if value.identifier.trim().is_empty() {
            return Err(ConnectionError::EmptyVersions);
        }
        for feature in value.features.iter() {
            if feature.trim().is_empty() {
                return Err(ConnectionError::EmptyFeatures);
            }
        }
        Ok(Version {
            identifier: value.identifier,
            features: value.features,
        })
    }
}

impl From<Version> for RawVersion {
    fn from(value: Version) -> Self {
        Self {
            identifier: value.identifier,
            features: value.features,
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Version {{ identifier: {}, features: {} }}",
            self.identifier,
            PrettySlice(&self.features)
        )
    }
}

/// Iterates over the descending ordered set of compatible IBC versions and
/// selects the first version with a version identifier that is supported by the
/// counterparty. The returned version contains a feature set with the
/// intersection of the features supported by the source and counterparty
/// chains. If the feature set intersection is nil then the search for a
/// compatible version continues. This function is called in the `conn_open_try`
/// handshake procedure.
///
/// NOTE: Empty feature set is not currently allowed for a chosen version.
pub fn pick_version(
    supported_versions: &[Version],
    counterparty_versions: &[Version],
) -> Result<Version, ConnectionError> {
    let mut intersection: Vec<Version> = Vec::new();
    for sv in supported_versions.iter() {
        if let Ok(cv) = find_supported_version(sv, counterparty_versions) {
            if let Ok(feature_set) = get_feature_set_intersection(&sv.features, &cv.features) {
                intersection.push(Version {
                    identifier: cv.identifier,
                    features: feature_set,
                })
            }
        }
    }

    if intersection.is_empty() {
        return Err(ConnectionError::NoCommonVersion);
    }

    intersection.sort_by(|a, b| a.identifier.cmp(&b.identifier));
    Ok(intersection[0].clone())
}

/// Returns the version from the list of supported versions that matches the
/// given reference version.
fn find_supported_version(
    version: &Version,
    supported_versions: &[Version],
) -> Result<Version, ConnectionError> {
    supported_versions
        .iter()
        .find(|sv| sv.identifier == version.identifier)
        .ok_or(ConnectionError::VersionNotSupported {
            version: version.clone(),
        })
        .cloned()
}

/// Returns the intersections of supported features by a host and the
/// counterparty features. This is done by iterating over all the features in
/// the host supported version and seeing if they exist in the feature set for
/// the counterparty version.
fn get_feature_set_intersection(
    supported_features: &[String],
    counterparty_features: &[String],
) -> Result<Vec<String>, ConnectionError> {
    let feature_set_intersection: Vec<String> = supported_features
        .iter()
        .filter(|f| counterparty_features.contains(f))
        .cloned()
        .collect();

    if feature_set_intersection.is_empty() {
        return Err(ConnectionError::NoCommonFeatures);
    }

    Ok(feature_set_intersection)
}

#[cfg(test)]
mod tests {
    use ibc_primitives::prelude::*;
    use ibc_proto::ibc::core::connection::v1::Version as RawVersion;

    use crate::error::ConnectionError;
    use crate::version::{pick_version, Version};

    fn get_dummy_features() -> Vec<String> {
        vec!["ORDER_RANDOM".to_string(), "ORDER_UNORDERED".to_string()]
    }

    fn good_versions() -> Vec<RawVersion> {
        vec![
            Version {
                identifier: "1".to_string(),
                features: vec!["ORDER_ORDERED".to_string(), "ORDER_UNORDERED".to_string()],
            }
            .into(),
            RawVersion {
                identifier: "2".to_string(),
                features: get_dummy_features(),
            },
        ]
        .into_iter()
        .collect()
    }

    fn bad_versions_identifier() -> Vec<RawVersion> {
        vec![RawVersion {
            identifier: "".to_string(),
            features: get_dummy_features(),
        }]
        .into_iter()
        .collect()
    }

    fn bad_versions_features() -> Vec<RawVersion> {
        vec![RawVersion {
            identifier: "2".to_string(),
            features: vec!["".to_string()],
        }]
        .into_iter()
        .collect()
    }

    fn overlapping() -> (Vec<Version>, Vec<Version>, Version) {
        (
            vec![
                Version {
                    identifier: "1".to_string(),
                    features: vec!["ORDER_ORDERED".to_string(), "ORDER_UNORDERED".to_string()],
                },
                Version {
                    identifier: "3".to_string(),
                    features: get_dummy_features(),
                },
                Version {
                    identifier: "4".to_string(),
                    features: get_dummy_features(),
                },
            ]
            .into_iter()
            .collect(),
            vec![
                Version {
                    identifier: "2".to_string(),
                    features: get_dummy_features(),
                },
                Version {
                    identifier: "4".to_string(),
                    features: get_dummy_features(),
                },
                Version {
                    identifier: "3".to_string(),
                    features: get_dummy_features(),
                },
            ]
            .into_iter()
            .collect(),
            // Should pick version 3 as it's the lowest of the intersection {3, 4}
            Version {
                identifier: "3".to_string(),
                features: get_dummy_features(),
            },
        )
    }

    fn disjoint() -> (Vec<Version>, Vec<Version>) {
        (
            vec![Version {
                identifier: "1".to_string(),
                features: Vec::new(),
            }]
            .into_iter()
            .collect(),
            vec![Version {
                identifier: "2".to_string(),
                features: Vec::new(),
            }]
            .into_iter()
            .collect(),
        )
    }

    #[test]
    fn verify() {
        struct Test {
            name: String,
            versions: Vec<RawVersion>,
            want_pass: bool,
        }
        let tests: Vec<Test> = vec![
            Test {
                name: "Compatible versions".to_string(),
                versions: vec![Version {
                    identifier: "1".to_string(),
                    features: get_dummy_features(),
                }
                .into()],
                want_pass: true,
            },
            Test {
                name: "Multiple versions".to_string(),
                versions: good_versions(),
                want_pass: true,
            },
            Test {
                name: "Bad version identifier".to_string(),
                versions: bad_versions_identifier(),
                want_pass: false,
            },
            Test {
                name: "Bad version feature".to_string(),
                versions: bad_versions_features(),
                want_pass: false,
            },
            Test {
                name: "Bad versions empty".to_string(),
                versions: Vec::new(),
                want_pass: true,
            },
        ];

        for test in tests {
            let versions = test
                .versions
                .into_iter()
                .map(Version::try_from)
                .collect::<Result<Vec<_>, _>>();

            assert_eq!(
                test.want_pass,
                versions.is_ok(),
                "Validate versions failed for test {} with error {:?}",
                test.name,
                versions.err(),
            );
        }
    }
    #[test]
    fn pick() {
        struct Test {
            name: String,
            supported: Vec<Version>,
            counterparty: Vec<Version>,
            picked: Result<Version, ConnectionError>,
            want_pass: bool,
        }
        let tests: Vec<Test> = vec![
            Test {
                name: "Compatible versions".to_string(),
                supported: Version::compatibles(),
                counterparty: Version::compatibles(),
                picked: Ok(Version {
                    identifier: "1".to_string(),
                    features: vec!["ORDER_ORDERED".to_string(), "ORDER_UNORDERED".to_string()],
                }),
                want_pass: true,
            },
            Test {
                name: "Overlapping versions".to_string(),
                supported: overlapping().0,
                counterparty: overlapping().1,
                picked: Ok(overlapping().2),
                want_pass: true,
            },
            Test {
                name: "Disjoint versions".to_string(),
                supported: disjoint().0,
                counterparty: disjoint().1,
                picked: Err(ConnectionError::NoCommonVersion),
                want_pass: false,
            },
        ];

        for test in tests {
            let version = pick_version(&test.supported, &test.counterparty);

            assert_eq!(
                test.want_pass,
                version.is_ok(),
                "Validate versions failed for test {}",
                test.name,
            );

            if test.want_pass {
                assert_eq!(version.unwrap(), test.picked.unwrap());
            }
        }
    }
    #[test]
    fn serialize() {
        let def = Version {
            identifier: "1".to_string(),
            features: vec!["ORDER_ORDERED".to_string(), "ORDER_UNORDERED".to_string()],
        };
        let def_raw: RawVersion = def.clone().into();
        let def_back = def_raw.try_into().unwrap();
        assert_eq!(def, def_back);
    }
}
