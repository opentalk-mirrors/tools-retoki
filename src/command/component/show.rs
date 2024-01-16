// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{fs::File, path::Path};

use anyhow::{Context, Result};
use clap::Args;
use serde::Serialize;
use tabled::Tabled;

use crate::{
    data::{self, ComponentIdentifier, ComponentName},
    output_format::OutputFormat,
};

#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct ShowArgs {
    /// The format in which to print the information
    #[clap(long, default_value = "table")]
    format: OutputFormat,
}

impl ShowArgs {
    pub fn execute<R: AsRef<Path>>(
        self,
        release_file: R,
        identifier: &ComponentIdentifier,
    ) -> Result<()> {
        let Self { format } = self;
        let file = File::open(&release_file).context(format!(
            "Couldn't open release file {:?}",
            release_file.as_ref()
        ))?;
        let raw_data: data::Releases = serde_yaml::from_reader(file)?;

        let component = raw_data
            .components
            .get(identifier)
            .context(format!("Component {identifier} not found"))?;

        #[derive(Debug, Serialize, Tabled)]
        struct ComponentInformation<'a> {
            #[tabled(rename = "Version")]
            pub name: &'a ComponentName,

            #[tabled(rename = "GitLab URL")]
            pub gitlab_url: &'a String,

            #[tabled(
                rename = "Container base URL",
                display_with = "crate::helper::tabled::display_option"
            )]
            #[serde(skip_serializing_if = "Option::is_none")]
            pub container_base_url: &'a Option<String>,
        }

        let info = ComponentInformation {
            name: &component.name,
            gitlab_url: &component.gitlab_url,
            container_base_url: &component.container_base_url,
        };
        format.output(&info)?;

        Ok(())
    }
}
