use self::check_milestones::CheckMilestonesArgs;
use crate::Config;
use clap::Subcommand;
use snafu::Whatever;

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
