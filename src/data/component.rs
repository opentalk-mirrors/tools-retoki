// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use indexmap::IndexMap;
use semver::Version;
use serde::{Deserialize, Serialize};

use super::{
    releases::StripReleases, ComponentCategoryIdentifier, ComponentName, ComponentRelease,
};
use crate::helper::releases::is_obsolete_prerelease;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Component {
    pub name: ComponentName,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub gitlab_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_base_url: Option<String>,
    pub category: ComponentCategoryIdentifier,

    #[serde(default)]
    pub releases: IndexMap<Version, ComponentRelease>,
}

impl Component {
    pub fn get_releases(
        &self,
        after: Option<Version>,
        until: Version,
    ) -> BTreeMap<Version, ComponentRelease> {
        if let Some(after) = after {
            self.releases
                .iter()
                .filter_map(|(v, r)| {
                    if *v > after && *v <= until {
                        Some((v.clone(), r.clone()))
                    } else {
                        None
                    }
                })
                .collect::<BTreeMap<Version, ComponentRelease>>()
        } else {
            self.releases
                .iter()
                .filter_map(|(v, r)| {
                    if *v <= until {
                        Some((v.clone(), r.clone()))
                    } else {
                        None
                    }
                })
                .collect::<BTreeMap<Version, ComponentRelease>>()
        }
    }

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
