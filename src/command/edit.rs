// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::Path;

use anyhow::Result;
use clap::Args;
use owo_colors::OwoColorize;

use crate::data::{read_release_file, write_releases_file, ReleaseFileReadOptions, StripReleases};

#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct EditArgs {
    /// Strip prereleases from the `releases.yml` file
    #[clap(long)]
    pub strip_prereleases: bool,
}

impl EditArgs {
    pub fn execute<R: AsRef<Path>>(self, release_file: R) -> Result<()> {
        let raw_data = read_release_file(
            &release_file,
            ReleaseFileReadOptions {
                strip_prereleases: self.strip_prereleases.then_some(StripReleases::PreReleases),
                ..Default::default()
            },
        )?;

        write_releases_file(&release_file, raw_data)?;

        println!();
        println!(
            "Release file {} has been {}",
            release_file.as_ref().to_string_lossy().bold(),
            "updated".green()
        );

        Ok(())
    }
}
