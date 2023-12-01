// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::Path;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::data::ComponentIdentifier;

mod show;

#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct ComponentArgs {
    /// The release version
    pub component: ComponentIdentifier,

    #[clap(subcommand)]
    pub command: ComponentCommand,
}

impl ComponentArgs {
    pub fn execute<R: AsRef<Path>>(self, release_file: R) -> Result<()> {
        let ComponentArgs { component, command } = self;
        command.execute(release_file, &component)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Subcommand)]
pub enum ComponentCommand {
    Show(show::ShowArgs),
}

impl ComponentCommand {
    pub fn execute<R: AsRef<Path>>(
        self,
        release_file: R,
        component: &ComponentIdentifier,
    ) -> Result<()> {
        match self {
            ComponentCommand::Show(args) => args.execute(release_file, component),
        }
    }
}
