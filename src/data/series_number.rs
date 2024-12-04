// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use core::fmt;
use std::{fmt::Display, str::FromStr};

use semver::{BuildMetadata, Prerelease, Version};
use serde::{de, Serialize, Serializer};

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::From, derive_more::Into,
)]
pub struct SeriesNumber {
    major: u64,
    minor: u64,
}

impl SeriesNumber {
    pub fn to_semver_version(
        &self,
        patch: u64,
        pre: Option<Prerelease>,
        build: Option<BuildMetadata>,
    ) -> Version {
        Version {
            major: self.major,
            minor: self.minor,
            patch,
            pre: pre.unwrap_or_default(),
            build: build.unwrap_or_default(),
        }
    }
}

impl Display for SeriesNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl From<Version> for SeriesNumber {
    fn from(value: Version) -> Self {
        Self::from(&value)
    }
}

impl From<&Version> for SeriesNumber {
    fn from(value: &Version) -> Self {
        Self {
            major: value.major,
            minor: value.minor,
        }
    }
}

impl FromStr for SeriesNumber {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (major, minor) = s
            .split_once('.')
            .ok_or("Couldn't find '.' delimiter inside series version number".to_string())?;

        let major = major
            .parse::<u64>()
            .map_err(|_| "First part of series number must be an integer number".to_string())?;

        let minor = minor
            .parse::<u64>()
            .map_err(|_| "First part of series number must be an integer number".to_string())?;

        Ok(Self { major, minor })
    }
}

impl<'de> de::Deserialize<'de> for SeriesNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct SeriesNumberVisitor;

        impl de::Visitor<'_> for SeriesNumberVisitor {
            type Value = SeriesNumber;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("SeriesNumber")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                SeriesNumber::from_str(v).map_err(|e| E::custom(e))
            }
        }

        deserializer.deserialize_str(SeriesNumberVisitor)
    }
}

impl Serialize for SeriesNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
