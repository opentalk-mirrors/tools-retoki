// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use clap::Subcommand;

use self::ci::CiArgs;
use crate::{cli::CommonArgs, Config};

mod ci;

#[derive(Clone, Debug, Subcommand)]
pub(crate) enum Command {
    /// Run all tasks that should be executed in a CI run
    Ci(CiArgs),
}

impl Command {
    pub(crate) fn run(&self, common_args: &CommonArgs, config: &Config) -> anyhow::Result<()> {
        match self {
            Self::Ci(args) => args.run(common_args, config),
        }
    }
}
