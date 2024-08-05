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
