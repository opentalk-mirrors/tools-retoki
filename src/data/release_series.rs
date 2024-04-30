// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use indexmap::IndexMap;
use semver::Version;
use serde::{Deserialize, Serialize};
use time::Date;

use super::{releases::StripReleases, Release, SeriesCodename};
use crate::helper::releases::is_obsolete_prerelease;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseSeries {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codename: Option<SeriesCodename>,
    pub end_of_life: Date,
    pub releases: IndexMap<Version, Release>,
}

impl ReleaseSeries {
    pub fn with_releases_stripped(self, strip_releases: StripReleases) -> Self {
        let releases = self
            .releases
            .clone()
            .into_iter()
            .filter(|(version, _release)| match strip_releases {
                StripReleases::ObsoletePreReleases => {
                    !is_obsolete_prerelease(&self.releases, version)
                }
                StripReleases::PreReleases => version.pre.is_empty(),
            })
            .collect();
        Self { releases, ..self }
    }
}
