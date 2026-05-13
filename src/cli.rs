// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use clap::{Args, Parser};

use crate::{Command, Config};

#[derive(Clone, Debug, Parser)]
#[command(version, about)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[command(flatten)]
    pub common: CommonArgs,
}

#[derive(Clone, Debug, Args)]
pub(crate) struct CommonArgs {
    #[arg(long, env = "RELBO_DRY_RUN", global = true)]
    pub dry_run: bool,
}

impl Cli {
    pub(crate) fn run(&self, config: &Config) -> anyhow::Result<()> {
        self.command.run(&self.common, config)
    }
}
