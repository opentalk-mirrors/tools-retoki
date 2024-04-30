// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{fs::File, path::Path};

use anyhow::{Context as _, Result};
use clap::Args;
use owo_colors::OwoColorize;

use super::utils::write_releases_yml_file;
use crate::data::{self, StripReleases};

#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct EditArgs {
    /// Strip prereleases from the `releases.yml` file
    #[clap(long)]
    pub strip_prereleases: bool,
}

impl EditArgs {
    pub fn execute<R: AsRef<Path>>(self, release_file: R) -> Result<()> {
        let file = File::open(&release_file).context(format!(
            "Couldn't open release file {:?}",
            release_file.as_ref()
        ))?;

        let mut raw_data: data::Releases = serde_yaml::from_reader::<_, data::Releases>(file)?;

        if self.strip_prereleases {
            raw_data = raw_data.with_releases_stripped(StripReleases::PreReleases);
        }

        write_releases_yml_file(&release_file, raw_data)?;

        println!();
        println!(
            "Release file {} has been {}",
            release_file.as_ref().to_string_lossy().bold(),
            "updated".green()
        );

        Ok(())
    }
}
