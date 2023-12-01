// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::data::{self, ProductName};

use super::{Component, EmptyReleaseComponent, ReleaseSeries};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Readme {
    pub product_name: ProductName,
    pub series: Vec<ReleaseSeries>,
    pub components: Vec<Component>,

    // TODO: this is an ugly workaround to get beautiful spaciing for tables,
    // because tera whitespace control appears to not be providing what is needed
    // to control the number of spaces in the loop elements properly
    pub space: String,

    // TODO: this is an ugly workaround to get an empty dummy release version
    // into the components, as it looks like one can't create object values
    // inside tera
    pub empty_release_component: EmptyReleaseComponent,
}

impl Readme {
    pub fn from_data_releases(releases: &data::Releases) -> Result<Self> {
        Ok(Self {
            product_name: releases.product_name.clone(),
            series: releases
                .series
                .iter()
                .map(|(version, series)| {
                    ReleaseSeries::from_data_release_series(
                        version.clone(),
                        series,
                        &releases.components,
                    )
                })
                .collect::<Result<_, _>>()?,
            components: releases
                .components
                .iter()
                .map(|(identifier, component)| {
                    Component::from_data_component(identifier.clone(), component)
                })
                .collect(),
            space: " ".to_string(),
            empty_release_component: EmptyReleaseComponent::default(),
        })
    }
}
