// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use indexmap::IndexMap;
use semver::Version;
use serde::{Deserialize, Serialize};
use time::Date;

use super::{Release, releases::StripReleases};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseSeries {
    pub end_of_life: Date,
    pub releases: IndexMap<Version, Release>,
}

impl ReleaseSeries {
    pub fn with_releases_stripped(self, strip_releases: StripReleases) -> Self {
        let all_releases: BTreeSet<Version> = self.releases.keys().cloned().collect();

        let releases = self
            .releases
            .clone()
            .into_iter()
            .filter(|(version, _release)| match strip_releases {
                StripReleases::ObsoletePreReleases => {
                    !strip_releases.is_obsolete_prerelease(&all_releases, version)
                }
                StripReleases::PreReleases => !strip_releases.is_prerelease(version),
            })
            .collect();
        Self { releases, ..self }
    }
}
