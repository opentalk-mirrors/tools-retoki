// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::Path;

use anyhow::Result;
use clap::{Args, Subcommand};
use semver::Version;

mod show;

#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct ReleaseArgs {
    /// The release version
    pub version: Version,

    #[clap(subcommand)]
    pub command: ReleaseCommand,
}

impl ReleaseArgs {
    pub fn execute<R: AsRef<Path>>(self, release_file: R) -> Result<()> {
        let ReleaseArgs { version, command } = self;
        command.execute(release_file, &version)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Subcommand)]
pub enum ReleaseCommand {
    Show(show::ShowArgs),
}

impl ReleaseCommand {
    pub fn execute<R: AsRef<Path>>(self, release_file: R, version: &Version) -> Result<()> {
        match self {
            ReleaseCommand::Show(args) => args.execute(release_file, version),
        }
    }
}
