// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use clap::Parser;
use command::Command;

mod command;
mod data;
mod output_format;
mod template;

#[derive(Clone, Debug, PartialEq, Eq, Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.command.execute()
}
