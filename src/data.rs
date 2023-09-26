// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use semver::Version;
use serde::{Deserialize, Serialize};
use time::Date;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Releases {
    pub product_name: String,
    pub components: BTreeMap<String, Component>,
    pub series: BTreeMap<String, ReleaseSeries>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub gitlab_url: String,

    #[serde(default)]
    pub releases: BTreeMap<Version, ComponentRelease>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentRelease {
    pub changelog: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseSeries {
    pub codename: String,
    pub end_of_life: Date,
    pub releases: BTreeMap<Version, Release>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Release {
    pub date: Date,
    pub components: BTreeMap<String, Version>,
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
}
