// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::PathBuf;

use anyhow::Result;
use clap::Subcommand;
use semver::Version;
use time::{Date, OffsetDateTime};

use crate::output_format::OutputFormat;

use self::utils::parse_date;

mod generate;
mod list;
mod show_release;
mod utils;

#[derive(Clone, Debug, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// Generate the release information from a `releases.yml` file.
    Generate {
        /// The YAML file containing the structured release information.
        #[clap(long, default_value = "releases.yml")]
        release_file: PathBuf,

        /// The target directory for the release information.
        #[clap(long, default_value = ".")]
        target_dir: PathBuf,
    },

    /// List the release series
    /// The series are always ordered ascending by their number.
    ListSeries {
        /// The YAML file containing the structured release information.
        #[clap(long, default_value = "releases.yml")]
        release_file: PathBuf,

        /// The format in which to print the information
        #[clap(long, default_value = "table")]
        format: OutputFormat,

        /// The date that is used as the basis for calculating EOL values
        #[clap(long, value_parser = parse_date)]
        date: Option<Date>,
    },

    /// Show details about a release
    ShowRelease {
        /// The series for which to print the information.
        version: Version,

        /// The YAML file containing the structured release information.
        #[clap(long, default_value = "releases.yml")]
        release_file: PathBuf,

        /// The format in which to print the information
        #[clap(long, default_value = "table")]
        format: OutputFormat,
    },
}

impl Command {
    pub fn execute(self) -> Result<()> {
        match self {
            Command::Generate {
                release_file,
                target_dir,
            } => generate::execute(release_file, target_dir),
            Command::ListSeries {
                release_file,
                format,
                date,
            } => list::execute(
                release_file,
                format,
                date.unwrap_or_else(|| OffsetDateTime::now_utc().date()),
            ),
            Command::ShowRelease {
                version,
                release_file,
                format,
            } => show_release::execute(&version, release_file, format),
        }
    }
}
