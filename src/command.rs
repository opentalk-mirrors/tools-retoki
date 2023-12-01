// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Subcommand;

use self::{component::ComponentArgs, release::ReleaseArgs, series::SeriesArgs};

mod component;
mod generate;
mod release;
mod series;
mod utils;

#[derive(Clone, Debug, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// Generate the release information from a `releases.yml` file.
    Generate {
        /// The target directory for the release information.
        #[clap(long, default_value = ".")]
        target_dir: PathBuf,
    },

    /// Perform actions releated a release
    Release(ReleaseArgs),

    /// Perform actions releated a release series
    Series(SeriesArgs),

    /// Perform actions releated a component
    Component(ComponentArgs),
}

impl Command {
    pub fn execute<R: AsRef<Path>>(self, release_file: R) -> Result<()> {
        match self {
            Command::Generate { target_dir } => generate::execute(release_file, target_dir),
            Command::Release(args) => args.execute(release_file),
            Command::Component(args) => args.execute(release_file),
            Command::Series(args) => args.execute(release_file),
        }
    }
}
