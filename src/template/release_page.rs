// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context as _, Result};
use semver::Version;
use serde::{Deserialize, Serialize};
use time::Date;

use super::{Release, ReleaseSeries};
use crate::data::{self, ProductName, SeriesNumber};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleasePage {
    pub product_name: ProductName,

    #[serde(flatten)]
    pub release: Release,

    pub series: ReleaseSeries,

    // TODO: this is an ugly workaround to get beautiful spaciing for tables,
    // because tera whitespace control appears to not be providing what is needed
    // to control the number of spaces in the loop elements properly
    pub space: String,

    pub show_gitlab_release_links: bool,

    pub show_md_header: bool,
}

impl ReleasePage {
    #[allow(clippy::too_many_arguments)]
    pub fn from_data_release(
        releases: &data::Releases,
        profile: &data::Profile,
        version: Version,
        previous: Option<(Version, &data::Release)>,
        next: Option<(Version, &data::Release)>,
        end_date: Date,
        show_md_header: bool,
        date: Date,
    ) -> Result<Self> {
        let series_number = SeriesNumber::from(&version);

        let changelog_base = previous
            .as_ref()
            .map(|(k, v)| (k.clone(), (*v).clone()))
            .or_else(|| {
                let previous_series = releases
                    .series
                    .iter()
                    .rev()
                    .find(|(k, _)| **k < series_number);
                let releases = previous_series
                    .map(|s| s.1.releases.clone())
                    .unwrap_or_default();
                releases
                    .iter()
                    .rev()
                    .find(|v| v.0.pre.is_empty())
                    .or_else(|| releases.iter().next_back())
                    .map(|(k, v)| (k.clone(), v.clone()))
            });

        let series = releases
            .series
            .get(&series_number)
            .with_context(|| format!("Couldn't find release series {series_number:?}."))?;

        let release = series.releases.get(&version).with_context(|| {
            format!("Couldn't find release {version:?} in series {series_number}.")
        })?;

        Ok(Self {
            product_name: releases.product_name.clone(),
            series: ReleaseSeries::from_data_release_series(
                series_number,
                series,
                &releases.components,
                &profile.components,
                &releases.component_categories,
                date,
            )?,
            release: Release::from_data_release(
                version,
                changelog_base,
                previous,
                next,
                end_date,
                release,
                &releases.components,
                &profile.components,
                &releases.component_categories,
            )?,
            space: " ".to_string(),
            show_gitlab_release_links: true,
            show_md_header,
        })
    }
}
