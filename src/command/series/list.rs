// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::Path;

use anyhow::Result;
use clap::Args;
use semver::Version;
use serde::Serialize;
use tabled::{Tabled, derive::display};
use time::{Date, OffsetDateTime};

use crate::{
    command::utils::parse_date,
    data::{SeriesNumber, read_release_file},
    output_format::OutputFormat,
};

#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct ListArgs {
    /// The format in which to print the information
    #[clap(long, default_value = "table")]
    format: OutputFormat,

    /// The date that is used as the basis for calculating EOL values
    #[clap(long, value_parser = parse_date)]
    date: Option<Date>,
}

impl ListArgs {
    pub fn execute<R: AsRef<Path>>(self, release_file: R) -> Result<()> {
        let Self { format, date } = self;

        let date = date.unwrap_or_else(|| OffsetDateTime::now_utc().date());

        let raw_data = read_release_file(release_file)?;

        #[derive(Debug, Serialize, Tabled)]
        #[tabled(display(Option, "display::option", "-"))]
        struct SeriesInformation {
            #[tabled(rename = "Series number")]
            pub series_number: SeriesNumber,

            #[tabled(rename = "Highest release")]
            pub highest_release: Option<Version>,

            #[tabled(rename = "EOL")]
            pub is_eol: bool,
        }

        let mut all_series = Vec::new();

        for (series_number, series) in raw_data.series {
            let highest_release = series.releases.into_keys().next_back();
            let is_eol = series.end_of_life < date;
            let info = SeriesInformation {
                series_number,
                highest_release,
                is_eol,
            };
            all_series.push(info);
        }
        format.output_multiple(&all_series)?;

        Ok(())
    }
}
