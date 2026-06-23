// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::Path;

use anyhow::{Context, Result};
use clap::Args;

use crate::data::{
    ReleaseFileReadOptions, Releases, StripReleases, read_release_file_with_options,
    write_releases_file,
};

#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct EditArgs {
    /// Strip prereleases from the `releases.yml` file
    #[clap(long)]
    pub strip_prereleases: bool,

    /// Strip all changelogs from the `releases.yml` file
    #[clap(long)]
    pub strip_changelogs: bool,
}

impl EditArgs {
    pub fn execute<R: AsRef<Path>>(self, release_file: R) -> Result<()> {
        let mut raw_data = read_release_file_with_options(
            &release_file,
            ReleaseFileReadOptions {
                strip_prereleases: self.strip_prereleases.then_some(StripReleases::PreReleases),
            },
        )
        .context("Failed to read release configuration")?;

        if self.strip_changelogs {
            strip_changelogs(&mut raw_data);
        }

        write_releases_file(&release_file, raw_data)
            .context("Failed to write release configuration")?;

        tracing::info!(
            release_file = %release_file.as_ref().display(),
            "Release file updated"
        );

        Ok(())
    }
}

fn strip_changelogs(releases: &mut Releases) {
    for (_, comp) in &mut releases.components {
        for (_, release) in &mut comp.releases {
            release.changelog.take();
        }
    }
}
