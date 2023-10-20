// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

mod component_identifier;
mod component_name;
mod product_name;
mod series_codename;
mod series_number;

use std::collections::BTreeMap;

use semver::Version;
use serde::{Deserialize, Serialize};
use time::Date;

pub use component_identifier::ComponentIdentifier;
pub use component_name::ComponentName;
pub use product_name::ProductName;
pub use series_codename::SeriesCodename;
pub use series_number::SeriesNumber;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Releases {
    pub product_name: ProductName,
    pub components: BTreeMap<ComponentIdentifier, Component>,
    pub series: BTreeMap<SeriesNumber, ReleaseSeries>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Component {
    pub name: ComponentName,
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
    pub codename: SeriesCodename,
    pub end_of_life: Date,
    pub releases: BTreeMap<Version, Release>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Release {
    pub date: Date,
    pub components: BTreeMap<ComponentIdentifier, Version>,
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
