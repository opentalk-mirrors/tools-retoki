// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::Path;

use anyhow::{Context, Result};
use clap::Args;
use serde::Serialize;
use tabled::Tabled;

use crate::{
    command::ProfileArgs,
    data::{read_profile_file, read_release_file, ComponentIdentifier, ComponentName},
    output_format::OutputFormat,
};

#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct ShowArgs {
    /// The format in which to print the information
    #[clap(long, default_value = "table")]
    format: OutputFormat,

    #[clap(flatten)]
    profile: ProfileArgs,
}

impl ShowArgs {
    pub fn execute<R: AsRef<Path>>(
        self,
        release_file: R,
        identifier: &ComponentIdentifier,
    ) -> Result<()> {
        let Self { format, profile } = self;
        let releases = read_release_file(&release_file)?;
        let profile = read_profile_file(
            &release_file,
            &profile.profile,
            profile.profile_path.as_deref(),
        )?;

        let component = releases
            .components
            .get(identifier)
            .with_context(|| format!("Component {identifier} not found"))?;
        let component_profile = profile.components.get(identifier).with_context(|| {
            format!(
                "Component {identifier} not found in profile {}",
                profile.profile_name
            )
        })?;
        #[derive(Debug, Serialize, Tabled)]
        struct ComponentInformation<'a> {
            #[tabled(rename = "Version")]
            pub name: &'a ComponentName,

            #[tabled(
                rename = "GitLab URL",
                display_with = "crate::helper::tabled::display_option"
            )]
            pub gitlab_url: &'a Option<String>,

            #[tabled(
                rename = "Container base URL",
                display_with = "crate::helper::tabled::display_option"
            )]
            #[serde(skip_serializing_if = "Option::is_none")]
            pub container_base_url: &'a Option<String>,
        }

        let info = ComponentInformation {
            name: &component.name,
            gitlab_url: &component_profile.gitlab_url,
            container_base_url: &component_profile.container_base_url,
        };
        format.output(&info)?;

        Ok(())
    }
}
