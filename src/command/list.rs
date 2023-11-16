// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{fs::File, path::Path};

use anyhow::{Context as _, Result};
use semver::Version;
use serde::Serialize;
use tabled::Tabled;
use time::Date;

use crate::{
    data::{self, SeriesNumber},
    output_format::OutputFormat,
};

use super::utils::tabled_display_option;

pub fn execute<R: AsRef<Path>>(release_file: R, format: OutputFormat, date: Date) -> Result<()> {
    let file = File::open(&release_file).context(format!(
        "Couldn't open release file {:?}",
        release_file.as_ref()
    ))?;
    let raw_data: data::Releases = serde_yaml::from_reader(file)?;

    #[derive(Debug, Serialize, Tabled)]
    struct SeriesInformation {
        #[tabled(rename = "Series number")]
        pub series_number: SeriesNumber,

        #[tabled(rename = "Highest release", display_with = "tabled_display_option")]
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
