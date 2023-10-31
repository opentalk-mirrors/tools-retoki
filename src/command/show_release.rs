// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::BTreeMap, fs::File, path::Path};

use anyhow::{Context, Result};
use semver::Version;
use serde::Serialize;
use tabled::Tabled;

use crate::{
    data::{self, ComponentIdentifier, SeriesNumber},
    output_format::OutputFormat,
};

pub fn execute<R: AsRef<Path>>(
    version: &Version,
    release_file: R,
    format: OutputFormat,
) -> Result<()> {
    let file = File::open(&release_file).context(format!(
        "Couldn't open release file {:?}",
        release_file.as_ref()
    ))?;
    let raw_data: data::Releases = serde_yaml::from_reader(file)?;

    let series_number = version.into();
    let series = raw_data
        .series
        .get(&series_number)
        .with_context(|| format!("Release series {series_number} not found"))?;
    let release = series
        .releases
        .get(version)
        .with_context(|| format!("Release {version} not found in series {series_number}"))?;

    #[derive(Debug, Serialize)]
    struct Component<'a> {
        identifier: &'a ComponentIdentifier,
        version: &'a Version,
    }

    fn display_components(components: &BTreeMap<ComponentIdentifier, Version>) -> String {
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
        pub components: &'a BTreeMap<ComponentIdentifier, Version>,
    }

    let info = ReleaseInformation {
        version,
        series: version.into(),
        components: &release.components,
    };
    format.output(&info)?;

    Ok(())
}
