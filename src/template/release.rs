// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context as _, Result};
use semver::Version;
use serde::{Deserialize, Serialize};
use time::Date;

use crate::data::{self, ComponentIdentifier};

use super::{ComponentRelease, ReleaseComponent};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Release {
    pub version: Version,
    pub previous: Option<Version>,
    pub next: Option<Version>,
    pub date: Date,
    pub end_date: Date,
    pub release_notes: Option<String>,
    pub components: Vec<ReleaseComponent>,
    pub components_by_identifier: BTreeMap<ComponentIdentifier, ReleaseComponent>,
    pub component_releases: BTreeMap<ComponentIdentifier, Vec<ComponentRelease>>,
}

impl Release {
    pub fn from_data_release(
        version: Version,
        changelog_base: Option<(Version, data::Release)>,
        previous: Option<(Version, &data::Release)>,
        next: Option<(Version, &data::Release)>,
        end_date: Date,
        release: &data::Release,
        components: &BTreeMap<ComponentIdentifier, data::Component>,
    ) -> Result<Self> {
        let mut component_releases = BTreeMap::new();

        for (component_identifier, component_version) in &release.components {
            let previous = changelog_base
                .iter()
                .flat_map(|(_, release)| release.components.get(component_identifier))
                .next();

            if let Some(component) = components.get(component_identifier) {
                let releases = component
                    .get_releases(previous.cloned(), component_version.clone())
                    .into_iter()
                    .map(|(version, release)| {
                        ComponentRelease::from_data_component_release(
                            &version,
                            component.gitlab_url.clone(),
                            &release,
                            BTreeSet::default(),
                        )
                    })
                    .collect::<Vec<_>>();

                if !releases.is_empty() {
                    component_releases.insert(component_identifier.clone(), releases);
                }
            }
        }

        Ok(Self {
            version,
            previous: previous.map(|(v, _)| v.clone()),
            next: next.map(|(v, _)| v.clone()),
            date: release.date,
            end_date,
            release_notes: release.release_notes.clone(),
            components: release
                .components
                .iter()
                .map(|(identifier, version)| {
                    let component = components
                        .get(identifier)
                        .context(format!("Couldn't find component {:?}", identifier))?;
                    Ok(ReleaseComponent::from_data_component(
                        identifier.clone(),
                        version.clone(),
                        component.gitlab_url.clone(),
                    ))
                })
                .collect::<Result<_, anyhow::Error>>()?,
            components_by_identifier: release
                .components
                .iter()
                .map(|(identifier, version)| {
                    let component = components
                        .get(identifier)
                        .context(format!("Couldn't find component {:?}", identifier))?;
                    Ok((
                        identifier.clone(),
                        ReleaseComponent::from_data_component(
                            identifier.clone(),
                            version.clone(),
                            component.gitlab_url.clone(),
                        ),
                    ))
                })
                .collect::<Result<_, anyhow::Error>>()?,
            component_releases,
        })
    }
}
