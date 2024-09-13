// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use super::ComponentRelease;
use crate::data::{
    Component, ComponentIdentifier, ComponentName, ComponentProfile, ProductName, Releases,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentPage {
    pub product_name: ProductName,
    pub component_name: ComponentName,
    pub component_identifier: ComponentIdentifier,
    pub releases: Vec<ComponentRelease>,

    pub sidebar_position: usize,

    // TODO: this is an ugly workaround to get beautiful spacing for tables,
    // because tera whitespace control appears to not be providing what is needed
    // to control the number of spaces in the loop elements properly
    pub space: String,

    pub show_md_header: bool,
}

impl ComponentPage {
    pub fn from_data_component(
        component_identifier: &ComponentIdentifier,
        component: &Component,
        profile: &ComponentProfile,
        product_name: &ProductName,
        data_releases: &Releases,
        sidebar_position: usize,
        show_md_header: bool,
    ) -> Self {
        let mut releases = component.releases.clone();
        releases.sort_keys();
        Self {
            product_name: product_name.clone(),
            component_name: component.name.clone(),
            component_identifier: component_identifier.clone(),
            releases: releases
                .into_iter()
                .map(|(version, release)| {
                    let product_releases = data_releases
                        .get_product_releases_for_component_version(component_identifier, &version);
                    ComponentRelease::from_data_component_release(
                        &version,
                        profile.gitlab_url.clone(),
                        &release,
                        product_releases,
                    )
                })
                .collect(),
            space: " ".to_string(),
            sidebar_position,
            show_md_header,
        }
    }
}
