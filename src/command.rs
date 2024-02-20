// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::Path;

use anyhow::Result;
use clap::Subcommand;

use self::{
    component::ComponentArgs, edit::EditArgs, generate::GenerateArgs, release::ReleaseArgs,
    series::SeriesArgs,
};

mod component;
mod edit;
mod generate;
mod release;
mod series;
mod utils;

#[derive(Clone, Debug, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// Generate the release information from a `releases.yml` file
    Generate(GenerateArgs),

    /// Perform actions releated a release
    Release(ReleaseArgs),

    /// Perform actions releated a release series
    Series(SeriesArgs),

    /// Perform actions releated a component
    Component(ComponentArgs),

    /// Edit a `releases.yml` file
    Edit(EditArgs),
}

impl Command {
    pub fn execute<R: AsRef<Path>>(self, release_file: R) -> Result<()> {
        match self {
            Command::Generate(args) => args.execute(release_file),
            Command::Release(args) => args.execute(release_file),
            Command::Component(args) => args.execute(release_file),
            Command::Series(args) => args.execute(release_file),
            Command::Edit(args) => args.execute(release_file),
        }
    }
}
