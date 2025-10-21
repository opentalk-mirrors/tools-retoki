// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    data::{
        self, ComponentCategory, ComponentCategoryIdentifier, ComponentIdentifier, ProductName,
        SeriesNumber,
    },
    template::ReleaseSeries,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseSeriesPage {
    pub product_name: ProductName,

    #[serde(flatten)]
    pub release_series: ReleaseSeries,

    pub show_md_header: bool,

    pub show_series_end_of_life: bool,
}

impl ReleaseSeriesPage {
    pub fn from_data_release_series(
        version: SeriesNumber,
        product_name: ProductName,
        release_series: &data::ReleaseSeries,
        components: &IndexMap<ComponentIdentifier, data::Component>,
        component_profiles: &IndexMap<ComponentIdentifier, data::ComponentProfile>,
        component_categories: &IndexMap<ComponentCategoryIdentifier, ComponentCategory>,
    ) -> Result<Self> {
        let release_series = ReleaseSeries::from_data_release_series(
            version,
            release_series,
            components,
            component_profiles,
            component_categories,
        )?;
        Ok(Self {
            product_name,
            release_series,
            show_md_header: false,
            show_series_end_of_life: true,
        })
    }
}
