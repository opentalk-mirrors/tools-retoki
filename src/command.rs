// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Args, Subcommand};

use self::{
    compare::CompareArgs, component::ComponentArgs, edit::EditArgs, generate::GenerateArgs,
    release::ReleaseArgs, series::SeriesArgs,
};

pub mod compare;
pub mod component;
pub mod edit;
pub mod generate;
pub mod release;
pub mod series;
pub mod utils;

#[derive(Clone, Debug, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// Generate the release information from a `releases.yml` file
    Generate(GenerateArgs),

    /// Perform actions related to a release
    Release(ReleaseArgs),

    /// Perform actions related to a release series
    Series(SeriesArgs),

    /// Perform actions related to a component
    Component(ComponentArgs),

    /// Edit a `releases.yml` file
    Edit(EditArgs),

    /// Compare the `releases.yml` file with another `releases.yml` file
    ///
    /// This will print to stdout one line for each release that is found in either of the
    /// `releases.yml` files
    ///
    /// A release was added from the other to the current `releases.yml` file:
    /// `+ <version>`
    ///
    /// A release was removed from the other to the current `releases.yml` file:
    /// `- <version>`
    ///
    /// A release is present in both `releases.yml` files and was unchanged:
    /// `= <version>`
    ///
    /// A release is present in both `releases.yml` files and was changed:
    /// `~ <version>`
    #[clap(verbatim_doc_comment)]
    Compare(CompareArgs),
}

impl Command {
    pub fn execute<R: AsRef<Path>>(self, release_file: R) -> Result<()> {
        match self {
            Command::Generate(args) => args.execute(release_file),
            Command::Release(args) => args.execute(release_file),
            Command::Component(args) => args.execute(release_file),
            Command::Series(args) => args.execute(release_file),
            Command::Edit(args) => args.execute(release_file),
            Command::Compare(args) => args.execute(release_file),
        }
    }
}

/// Arguments that are shared by all commands
#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct ProfileArgs {
    /// The name of the profile that should be used for the release yaml
    #[clap(short = 'p', long = "profile", env("RETOKI_PROFILE"))]
    pub profile: String,

    /// The path to the directory containing the profile configurations. Defaults
    /// to the `retoki-profiles` folder which is expected in the same directory as the
    /// `releases.yml`
    #[clap(long = "profile-path")]
    pub profile_path: Option<PathBuf>,
}
