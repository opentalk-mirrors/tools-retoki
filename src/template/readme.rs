// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use time::Date;

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

    pub show_series_end_of_life: bool,

    pub show_gitlab_release_links: bool,

    pub show_md_header: bool,

    pub show_next_release: bool,

    pub relative_documentation_base_path: Option<String>,
}

impl Readme {
    pub fn from_data_releases(
        releases: &data::Releases,
        profile: &Profile,
        date: Date,
    ) -> Result<Self> {
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
                        date,
                    )
                })
                .collect::<Result<_, _>>()?,
            components: {
                let mut components = Vec::new();
                for (identifier, component) in releases.components.iter() {
                    let component_profile =
                        profile.components.get(identifier).with_context(|| {
                            format!(
                                "Missing `{}` component in `{}` profile",
                                identifier, profile.profile_name
                            )
                        })?;
                    if component_profile.private {
                        continue;
                    }
                    components.push(Component::from_data_component(
                        identifier.clone(),
                        component,
                        component_profile,
                    ));
                }
                components
            },
            space: " ".to_string(),
            empty_release_component: EmptyReleaseComponent::default(),
            show_gantt_chart: true,
            show_series_end_of_life: true,
            show_gitlab_release_links: true,
            show_md_header: false,
            show_next_release: false,
            relative_documentation_base_path: None,
        })
    }
}
