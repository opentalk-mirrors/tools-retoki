// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use crate::data::{self, ComponentIdentifier, ComponentName, ProductName};

use super::ComponentRelease;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentPage {
    pub product_name: ProductName,
    pub component_name: ComponentName,
    pub component_identifier: ComponentIdentifier,
    pub releases: Vec<ComponentRelease>,

    // TODO: this is an ugly workaround to get beautiful spaciing for tables,
    // because tera whitespace control appears to not be providing what is needed
    // to control the number of spaces in the loop elements properly
    pub space: String,
}

impl ComponentPage {
    pub fn from_data_component(
        component_identifier: &ComponentIdentifier,
        data: &data::Component,
        product_name: &ProductName,
        data_releases: &data::Releases,
    ) -> Self {
        Self {
            product_name: product_name.clone(),
            component_name: data.name.clone(),
            component_identifier: component_identifier.clone(),
            releases: data
                .releases
                .iter()
                .map(|(version, release)| {
                    let product_releases = data_releases
                        .get_product_releases_for_component_version(component_identifier, version);
                    ComponentRelease::from_data_component_release(
                        version,
                        data.gitlab_url.to_string(),
                        release,
                        product_releases,
                    )
                })
                .collect(),
            space: " ".to_string(),
        }
    }
}
