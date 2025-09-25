// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use command::Command;

mod command;
mod data;
mod helper;
mod output_format;
mod release_metadata;
mod template;

#[derive(Clone, Debug, PartialEq, Eq, Parser)]
#[command(author, version, about)]
struct Cli {
    /// The YAML file containing the structured release information.
    #[clap(long, default_value = "releases.yml", env = "RETOKI_RELEASE_FILE")]
    release_file: PathBuf,

    #[command(subcommand)]
    command: Command,
}

fn main() -> Result<()> {
    let Cli {
        release_file,
        command,
    } = Cli::parse();
    command.execute(release_file)
}
