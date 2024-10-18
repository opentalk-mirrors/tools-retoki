// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{Component, EmptyReleaseComponent, ReleaseSeries};
use crate::data::{self, ProductName, Profile};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Readme {
    pub product_name: ProductName,
    pub releases_page_header: Option<String>,
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

    pub show_gantt_chart: bool,

    pub show_gitlab_release_links: bool,

    pub show_md_header: bool,
}

impl Readme {
    pub fn from_data_releases(releases: &data::Releases, profile: &Profile) -> Result<Self> {
        Ok(Self {
            product_name: releases.product_name.clone(),
            releases_page_header: releases.releases_page_header.clone(),
            series: releases
                .series
                .iter()
                .map(|(version, series)| {
                    ReleaseSeries::from_data_release_series(
                        version.clone(),
                        series,
                        &releases.components,
                        &profile.components,
                        &releases.component_categories,
                    )
                })
                .collect::<Result<_, _>>()?,
            components: releases
                .components
                .iter()
                .map(|(identifier, component)| {
                    let profile = profile.components.get(identifier).with_context(|| {
                        format!(
                            "Missing `{}` component in `{}` profile",
                            identifier, profile.profile_name
                        )
                    })?;
                    Ok(Component::from_data_component(
                        identifier.clone(),
                        component,
                        profile,
                    ))
                })
                .collect::<anyhow::Result<Vec<_>>>()?,
            space: " ".to_string(),
            empty_release_component: EmptyReleaseComponent::default(),
            show_gantt_chart: true,
            show_gitlab_release_links: true,
            show_md_header: false,
        })
    }
}
