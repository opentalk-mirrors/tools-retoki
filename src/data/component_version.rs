// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::fmt::Display;

use semver::Version;
use serde::{Deserialize, Serialize};

use super::SeriesNumber;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ComponentVersion {
    Semver(Version),
    SeriesNumber(SeriesNumber),
    Other(String),
}

impl ComponentVersion {
    pub const fn as_semver(&self) -> Option<&Version> {
        if let Self::Semver(v) = self {
            return Some(v);
        }
        None
    }
}

impl Display for ComponentVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Semver(semver) => semver.fmt(f),
            Self::SeriesNumber(series_number) => series_number.fmt(f),
            Self::Other(other) => other.fmt(f),
        }
    }
}
