// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use anyhow::{Context, Result};
use semver::Version;
use serde::{Deserialize, Serialize};

use self::release_component_metadata::ReleaseComponentMetadata;
use crate::data::{self, ComponentIdentifier, SeriesNumber};

mod release_component_metadata;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseMetadata {
    pub version: Version,
    pub components: BTreeMap<ComponentIdentifier, ReleaseComponentMetadata>,
}

impl ReleaseMetadata {
    pub fn from_data_release(data: &data::Releases, version: Version) -> Result<Self> {
        let series_number = SeriesNumber::from(version.clone());
        let release_series = data.series.get(&series_number).with_context(|| {
            format!("Release series {series_number} for version {version} not found")
        })?;
        let release = release_series.releases.get(&version).with_context(|| {
            format!("Release {version} not found in release series {series_number}")
        })?;

        let components = release
            .components
            .clone()
            .into_iter()
            .map(|(component_identifier, version)| {
                let component = data
                    .components
                    .get(&component_identifier)
                    .with_context(|| format!("Component {component_identifier} not found"))?;
                let component_release = component.releases.get(&version).with_context(|| {
                    format!("Release {version} for component {component_identifier} not found")
                })?;
                Ok((
                    component_identifier,
                    ReleaseComponentMetadata {
                        version,
                        date: component_release.date,
                    },
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;

        Ok(Self {
            version,
            components,
        })
    }
}
