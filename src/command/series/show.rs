// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::Path;

use anyhow::{Result, bail};
use clap::Args;
use indexmap::IndexMap;
use semver::Version;
use serde::Serialize;
use tabled::{Tabled, derive::display};
use time::{Date, OffsetDateTime};

use crate::{
    command::utils::parse_date,
    data::{
        ComponentIdentifier, ComponentVersion, ReleaseFileReadOptions, SeriesNumber, StripReleases,
        read_release_file_with_options,
    },
    helper::tabled::display_components,
    output_format::OutputFormat,
};

#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct ShowArgs {
    series_number: SeriesNumber,

    /// The format in which to print the information
    #[clap(long, default_value = "table")]
    format: OutputFormat,

    /// The date that is used as the basis for calculating if releases are EOL
    #[clap(long, value_parser = parse_date)]
    date: Option<Date>,

    /// Don't list any prereleases.
    #[clap(long)]
    without_prereleases: bool,
}

impl ShowArgs {
    pub fn execute<R: AsRef<Path>>(self, release_file: R) -> Result<()> {
        let Self {
            series_number,
            format,
            date,
            without_prereleases,
        } = self;

        let date = date.unwrap_or_else(|| OffsetDateTime::now_utc().date());

        let strip_prereleases = without_prereleases.then_some(StripReleases::PreReleases);

        let mut raw_data = read_release_file_with_options(
            release_file,
            ReleaseFileReadOptions { strip_prereleases },
        )?;

        #[derive(Debug, Serialize, Tabled)]
        #[tabled(display(Option, "display::option", "-"))]
        struct SeriesInformation {
            #[tabled(rename = "Series number")]
            pub series_number: SeriesNumber,

            #[tabled(rename = "Highest release")]
            pub highest_release: Option<Version>,

            #[tabled(rename = "EOL")]
            pub is_eol: bool,

            #[tabled(rename = "Support end")]
            pub end_of_life: Date,

            #[tabled(rename = "Components", display = "display_components")]
            pub highest_release_components: IndexMap<ComponentIdentifier, ComponentVersion>,
        }

        let Some(series) = raw_data.series.remove(&series_number) else {
            bail!("No series with number {series_number} found");
        };

        let highest_release = series.releases.keys().next_back().cloned();
        let highest_release_components = highest_release
            .as_ref()
            .and_then(|version| series.releases.get(version))
            .map(|release| release.components.clone())
            .unwrap_or_default();
        let is_eol = series.end_of_life < date;
        let info = SeriesInformation {
            series_number,
            highest_release,
            is_eol,
            end_of_life: series.end_of_life,
            highest_release_components,
        };
        format.output(&info)?;

        Ok(())
    }
}
