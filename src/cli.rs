// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use clap::Parser;
use snafu::Whatever;

use crate::{Command, Config};

#[derive(Clone, Debug, Parser)]
#[command(version, about)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    pub(crate) fn run(&self, config: &Config) -> Result<(), Whatever> {
        self.command.run(config)
    }
}
