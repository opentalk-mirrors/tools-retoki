// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::PathBuf;

use anyhow::Result;
use clap::Subcommand;

mod generate;

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
}

impl Command {
    pub fn execute(self) -> Result<()> {
        match self {
            Command::Generate {
                release_file,
                target_dir,
            } => generate::execute(release_file, target_dir),
        }
    }
}
