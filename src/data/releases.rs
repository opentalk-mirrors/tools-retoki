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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Releases {
    pub product_name: ProductName,
    pub components: BTreeMap<ComponentIdentifier, Component>,
    pub series: BTreeMap<SeriesNumber, ReleaseSeries>,
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

    pub fn without_obsolete_prereleases(self) -> Self {
        Self {
            series: self
                .series
                .into_iter()
                .map(|(number, series)| (number, series.without_obsolete_prereleases()))
                .collect(),
            components: self
                .components
                .into_iter()
                .map(|(number, component)| (number, component.without_obsolete_prereleases()))
                .collect(),
            ..self
        }
    }
}
