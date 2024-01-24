// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context as _, Result};
use semver::Version;
use serde::{Deserialize, Serialize};
use time::Date;

use crate::data::{self, ProductName, SeriesNumber};

use super::{Release, ReleaseSeries};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
}

impl ReleasePage {
    pub fn from_data_release(
        data: &data::Releases,
        version: Version,
        previous: Option<(Version, &data::Release)>,
        next: Option<(Version, &data::Release)>,
        end_date: Date,
    ) -> Result<Self> {
        let series_number = SeriesNumber::from(&version);

        let changelog_base = previous
            .as_ref()
            .map(|(k, v)| (k.clone(), (*v).clone()))
            .or_else(|| {
                let previous_series = data.series.iter().rev().find(|(k, _)| **k < series_number);
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

        let series = data
            .series
            .get(&series_number)
            .context("Couldn't find release series {series_number:?}.")?;

        let release = series
            .releases
            .get(&version)
            .context("Couldn't find release {version:?} in series {series_number}.")?;

        Ok(Self {
            product_name: data.product_name.clone(),
            series: ReleaseSeries::from_data_release_series(
                series_number,
                series,
                &data.components,
                &data.component_categories,
            )?,
            release: Release::from_data_release(
                version,
                changelog_base,
                previous,
                next,
                end_date,
                release,
                &data.components,
                &data.component_categories,
            )?,
            space: " ".to_string(),
            show_gitlab_release_links: true,
        })
    }
}
