// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use indexmap::IndexMap;
use semver::Version;
use serde::{Deserialize, Serialize};

use super::{
    Component, ComponentCategory, ComponentCategoryIdentifier, ComponentIdentifier, ProductName,
    ReleaseSeries, SeriesNumber,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StripReleases {
    /// Strip obsolete pre-releases (those where a final release is available)
    ObsoletePreReleases,

    /// Strip all pre-releleases
    PreReleases,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Releases {
    pub product_name: ProductName,
    pub series: BTreeMap<SeriesNumber, ReleaseSeries>,
    pub components: IndexMap<ComponentIdentifier, Component>,
    pub component_categories: IndexMap<ComponentCategoryIdentifier, ComponentCategory>,
}

impl Releases {
    pub fn get_product_releases_for_component_version(
        &self,
        component: &ComponentIdentifier,
        component_version: &Version,
    ) -> BTreeSet<Version> {
        let mut product_versions = BTreeSet::new();
        for series in &self.series {
            for (product_version, release) in &series.1.releases {
                if matches!(release.components.get(component), Some(v) if v == component_version) {
                    product_versions.insert(product_version.clone());
                }
            }
        }

        product_versions
    }

    pub fn strip_release_series_codenames(&mut self) {
        self.series.values_mut().for_each(|series| {
            series.codename.take();
        });
    }

    pub fn with_releases_stripped(self, strip_releases: StripReleases) -> Self {
        Self {
            series: self
                .series
                .into_iter()
                .map(|(number, series)| (number, series.with_releases_stripped(strip_releases)))
                .filter(|(_number, series)| !series.releases.is_empty())
                .collect(),
            components: self
                .components
                .into_iter()
                .map(|(number, component)| {
                    (number, component.with_releases_stripped(strip_releases))
                })
                .collect(),
            ..self
        }
    }
}
