// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
use clap::{Args, Subcommand};
use semver::Version;

use crate::{cli::CommonArgs, command::release::init::InitArgs, config::Config};

mod init;

#[derive(Debug, Clone, Args)]
pub(crate) struct ReleaseArgs {
    /// Release version to operate on.
    pub version: Version,

    #[command(subcommand)]
    pub action: ReleaseAction,
}

impl ReleaseArgs {
    pub(crate) fn run(&self, common_args: &CommonArgs, config: &Config) -> anyhow::Result<()> {
        match &self.action {
            ReleaseAction::Init(args) => args.run(self, common_args, config),
        }
    }

    /// Title used for the product release issue.
    pub fn title(&self) -> String {
        format!("Release {}", self.version)
    }
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum ReleaseAction {
    /// Create the product release issue for `<version>`.
    Init(InitArgs),
}
