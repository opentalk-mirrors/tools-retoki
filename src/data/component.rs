// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use indexmap::IndexMap;
use semver::Version;
use serde::{Deserialize, Serialize};

use super::{
    releases::StripReleases, ComponentCategoryIdentifier, ComponentName, ComponentRelease,
    ComponentVersion,
};
use crate::helper::releases::is_obsolete_prerelease;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub name: ComponentName,

    pub category: ComponentCategoryIdentifier,

    #[serde(default)]
    pub releases: IndexMap<ComponentVersion, ComponentRelease>,
}

impl Component {
    pub fn insert_release(
        &mut self,
        version: ComponentVersion,
        component_release: ComponentRelease,
    ) {
        self.releases.insert(version, component_release);
        // sort in reverse order, highest version number first
        self.releases
            .sort_unstable_by(|k1, _v1, k2, _v2| k2.cmp(k1));
    }

    pub fn get_releases(
        &self,
        after: Option<ComponentVersion>,
        until: ComponentVersion,
    ) -> BTreeMap<ComponentVersion, ComponentRelease> {
        if let Some(after) = after {
            return self
                .releases
                .iter()
                .filter(|(v, _)| (*v > &after && *v <= &until))
                .map(|(v, r)| (v.clone(), r.clone()))
                .collect::<BTreeMap<ComponentVersion, ComponentRelease>>();
        }
        self.releases
            .iter()
            .filter(|(v, _)| (*v <= &until))
            .map(|(v, r)| (v.clone(), r.clone()))
            .collect::<BTreeMap<ComponentVersion, ComponentRelease>>()
    }

    pub fn with_releases_stripped(self, strip_releases: StripReleases) -> Self {
        let releases: BTreeSet<Version> = self
            .releases
            .keys()
            .filter_map(|v| v.as_semver())
            .cloned()
            .collect();

        let releases = self
            .releases
            .clone()
            .into_iter()
            .filter(|(version, _release)| match strip_releases {
                StripReleases::ObsoletePreReleases => {
                    if let ComponentVersion::Semver(version) = version {
                        return !is_obsolete_prerelease(&releases, version);
                    }
                    true
                }
                StripReleases::PreReleases => {
                    if let ComponentVersion::Semver(version) = version {
                        return version.pre.is_empty();
                    }
                    true
                }
            })
            .collect();
        Self { releases, ..self }
    }
}
