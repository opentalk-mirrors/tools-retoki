// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::Path;

use anyhow::Result;
use clap::{Args, Subcommand};

mod list;

#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct SeriesArgs {
    #[clap(subcommand)]
    pub command: SeriesCommand,
}

impl SeriesArgs {
    pub fn execute<R: AsRef<Path>>(self, release_file: R) -> Result<()> {
        let SeriesArgs { command } = self;
        command.execute(release_file)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Subcommand)]
pub enum SeriesCommand {
    List(list::ListArgs),
}

impl SeriesCommand {
    pub fn execute<R: AsRef<Path>>(self, release_file: R) -> Result<()> {
        match self {
            SeriesCommand::List(args) => args.execute(release_file),
        }
    }
}
