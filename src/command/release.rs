// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::Path;

use anyhow::Result;
use clap::{Args, Subcommand};
use semver::Version;

mod add;
pub mod announce;
mod fetch_changelogs;
mod fetch_product_tickets;
mod init;

pub use add::AddArgs;
pub use announce::AnnounceArgs;
pub use fetch_changelogs::FetchChangelogsArgs;
pub use fetch_product_tickets::FetchProductTicketsArgs;
pub use init::InitArgs;

#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct ReleaseArgs {
    /// The release version
    pub version: Version,

    #[clap(subcommand)]
    pub command: ReleaseCommand,
}

impl ReleaseArgs {
    pub fn execute<R: AsRef<Path>>(self, release_file: R) -> Result<()> {
        let ReleaseArgs { version, command } = self;
        command.execute(release_file, &version)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Subcommand)]
pub enum ReleaseCommand {
    /// Create or update the product release issue for a version in the release tickets project
    ///
    /// Requires the GITLAB_TOKEN environment variable to be set
    Init(InitArgs),

    /// Add a component release issue that blocks the product release issue for a version
    ///
    /// Requires the GITLAB_TOKEN environment variable to be set
    Add(AddArgs),

    /// Fetch all changelog entries for the component releases of a product release
    ///
    /// Requires the GITLAB_TOKEN environment variable to be set
    FetchChangelogs(FetchChangelogsArgs),

    /// Fetch the product tickets for a release from the product tickets repository
    ///
    /// Requires the GITLAB_TOKEN environment variable to be set
    FetchProductTickets(FetchProductTicketsArgs),

    /// Render a release announcement in Markdown for communication channels such as Matrix or email
    Announce(AnnounceArgs),
}

impl ReleaseCommand {
    pub fn execute<R: AsRef<Path>>(self, release_file: R, version: &Version) -> Result<()> {
        match self {
            ReleaseCommand::Init(args) => args.execute(version),
            ReleaseCommand::Add(args) => args.execute(version),
            ReleaseCommand::FetchChangelogs(args) => args.execute(release_file, version),
            ReleaseCommand::FetchProductTickets(args) => args.execute(release_file, version),
            ReleaseCommand::Announce(args) => args.execute(release_file, version),
        }
    }
}
