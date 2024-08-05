use clap::Args;
use snafu::Whatever;

use crate::config::Config;

#[derive(Clone, Debug, Args)]
pub(crate) struct CheckMilestonesArgs;

impl CheckMilestonesArgs {
    pub(crate) fn run(&self, _config: &Config) -> Result<(), Whatever> {
        todo!()
    }
}
