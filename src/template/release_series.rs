// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use time::Date;

use super::Release;
use crate::data::{
    self, ComponentCategoryIdentifier, ComponentIdentifier, SeriesCodename, SeriesNumber,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseSeries {
    pub version: SeriesNumber,
    pub codename: Option<SeriesCodename>,
    pub end_of_life: Date,
    pub releases: Vec<Release>,
    pub markdown_anchor: String,
}

impl ReleaseSeries {
    pub fn from_data_release_series(
        version: SeriesNumber,
        release_series: &data::ReleaseSeries,
        components: &IndexMap<ComponentIdentifier, data::Component>,
        component_categories: &IndexMap<ComponentCategoryIdentifier, data::ComponentCategory>,
    ) -> Result<Self> {
        let release_markdown_code = match &release_series.codename {
            Some(codename) => format!("{} ({})", version, codename),
            None => version.to_string(),
        };
        let markdown_anchor = release_markdown_code
            .replace(['(', ')', '.'], "")
            .replace(' ', "-")
            .to_lowercase();
        let releases_with_padding = std::iter::once(None)
            .chain(release_series.releases.iter().map(Some))
            .chain(std::iter::once(None))
            .collect::<Vec<_>>();

        Ok(Self {
            version: version.clone(),
            codename: release_series.codename.clone(),
            end_of_life: release_series.end_of_life,
            releases: releases_with_padding
                .windows(3)
                .map(|window| {
                    let previous = window[0].map(|(v, r)| (v.clone(), r));
                    let (version, release) = window[1].unwrap();
                    let next = window[2].map(|(v, r)| (v.clone(), r));
                    let end_date = next
                        .as_ref()
                        .map(|(v, _)| {
                            release_series
                                .releases
                                .get(v)
                                .unwrap_or_else(|| panic!("version {v} not found"))
                                .date
                        })
                        .unwrap_or(release_series.end_of_life);
                    Release::from_data_release(
                        version.clone(),
                        None,
                        previous,
                        next,
                        end_date,
                        release,
                        components,
                        component_categories,
                    )
                })
                .collect::<Result<_, anyhow::Error>>()?,
            markdown_anchor,
        })
    }
}
