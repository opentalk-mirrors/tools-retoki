// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{cmp::Ordering, fmt::Display};

use semver::Version;
use serde::{Deserialize, Serialize};

use super::{SeriesNumber, prerelease};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

    pub fn prefix(&self) -> &'static str {
        match self {
            Self::Semver(_) => "v",
            Self::SeriesNumber(_) => "v",
            Self::Other(_) => "",
        }
    }

    pub fn prefixed(&self) -> String {
        format!("{}{self}", self.prefix())
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

impl Ord for ComponentVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Semver(a), Self::Semver(b)) => prerelease::cmp(a, b),
            (Self::SeriesNumber(a), Self::SeriesNumber(b)) => a.cmp(b),
            (Self::Other(a), Self::Other(b)) => a.cmp(b),
            _ => variant_rank(self).cmp(&variant_rank(other)),
        }
    }
}

impl PartialOrd for ComponentVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn variant_rank(v: &ComponentVersion) -> u8 {
    match v {
        ComponentVersion::Semver(_) => 0,
        ComponentVersion::SeriesNumber(_) => 1,
        ComponentVersion::Other(_) => 2,
    }
}
