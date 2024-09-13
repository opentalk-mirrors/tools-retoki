// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::Path;

use anyhow::{Context, Result};
use clap::Args;
use indexmap::IndexMap;
use semver::Version;
use serde::Serialize;
use tabled::Tabled;

use crate::{
    data::{read_release_file, ComponentIdentifier, ComponentVersion, SeriesNumber},
    output_format::OutputFormat,
};

#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct ShowArgs {
    /// The format in which to print the information
    #[clap(long, default_value = "table")]
    format: OutputFormat,
}

impl ShowArgs {
    pub fn execute<R: AsRef<Path>>(self, release_file: R, version: &Version) -> Result<()> {
        let Self { format } = self;

        let raw_data = read_release_file(&release_file, Default::default())?;

        let series_number = version.into();
        let series = raw_data
            .series
            .get(&series_number)
            .with_context(|| format!("Release series {series_number} not found"))?;
        let release = series
            .releases
            .get(version)
            .with_context(|| format!("Release {version} not found in series {series_number}"))?;

        fn display_components(
            components: &IndexMap<ComponentIdentifier, ComponentVersion>,
        ) -> String {
            components
                .iter()
                .map(|(identifier, version)| format!("{identifier}: {version}"))
                .fold(String::new(), |a, b| {
                    if a.is_empty() {
                        b
                    } else {
                        a + ", " + b.as_str()
                    }
                })
        }

        #[derive(Debug, Serialize, Tabled)]
        struct ReleaseInformation<'a> {
            #[tabled(rename = "Version")]
            pub version: &'a Version,

            #[tabled(rename = "Series")]
            pub series: SeriesNumber,

            #[tabled(rename = "Components", display_with = "display_components")]
            pub components: &'a IndexMap<ComponentIdentifier, ComponentVersion>,
        }

        let info = ReleaseInformation {
            version,
            series: version.into(),
            components: &release.components,
        };
        format.output(&info)?;

        Ok(())
    }
}
