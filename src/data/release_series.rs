// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use indexmap::IndexMap;
use semver::Version;
use serde::{Deserialize, Serialize};
use time::Date;

use super::{Release, SeriesCodename};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseSeries {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codename: Option<SeriesCodename>,
    pub end_of_life: Date,
    pub releases: IndexMap<Version, Release>,
}

impl ReleaseSeries {
    pub fn without_obsolete_prereleases(self) -> Self {
        let releases = self
            .releases
            .clone()
            .into_iter()
            .filter(|(version, _release)| {
                let is_final = version.pre.is_empty();
                let final_exists = self.releases.contains_key(&Version::new(
                    version.major,
                    version.minor,
                    version.patch,
                ));

                is_final || !final_exists
            })
            .collect();
        Self { releases, ..self }
    }
}
