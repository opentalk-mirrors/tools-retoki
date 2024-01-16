// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use indexmap::IndexMap;
use semver::Version;
use serde::{Deserialize, Serialize};

use super::{ComponentCategoryIdentifier, ComponentName, ComponentRelease};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Component {
    pub name: ComponentName,
    pub gitlab_url: String,

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
