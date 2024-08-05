// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use clap::Subcommand;
use snafu::Whatever;

use self::check_milestones::CheckMilestonesArgs;
use crate::Config;

mod check_milestones;

#[derive(Clone, Debug, Subcommand)]
pub(crate) enum Command {
    /// Check the milestones in the organization for due releases
    CheckMilestones(CheckMilestonesArgs),
}

impl Command {
    pub(crate) fn run(&self, config: &Config) -> Result<(), Whatever> {
        match self {
            Self::CheckMilestones(args) => args.run(config),
        }
    }
}
