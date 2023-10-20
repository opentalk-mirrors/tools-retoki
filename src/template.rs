// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use anyhow::{Context as _, Result};
use semver::Version;
use serde::{Deserialize, Serialize};
use time::Date;

use crate::data::{
    self, ComponentIdentifier, ComponentName, ProductName, SeriesCodename, SeriesNumber,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Readme {
    pub product_name: ProductName,
    pub series: Vec<ReleaseSeries>,
    pub components: Vec<Component>,

    // TODO: this is an ugly workaround to get beautiful spaciing for tables,
    // because tera whitespace control appears to not be providing what is needed
    // to control the number of spaces in the loop elements properly
    pub space: String,

    // TODO: this is an ugly workaround to get an empty dummy release version
    // into the components, as it looks like one can't create object values
    // inside tera
    pub empty_release_component: EmptyReleaseComponent,
}

impl Readme {
    pub fn from_data_releases(releases: &data::Releases) -> Result<Self> {
        Ok(Self {
            product_name: releases.product_name.clone(),
            series: releases
                .series
                .iter()
                .map(|(version, series)| {
                    ReleaseSeries::from_data_release_series(
                        version.clone(),
                        series,
                        &releases.components,
                    )
                })
                .collect::<Result<_, _>>()?,
            components: releases
                .components
                .iter()
                .map(|(identifier, component)| {
                    Component::from_data_component(identifier.clone(), component)
                })
                .collect(),
            space: " ".to_string(),
            empty_release_component: EmptyReleaseComponent::default(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleasePage {
    pub product_name: ProductName,

    #[serde(flatten)]
    pub release: Release,

    pub series: ReleaseSeries,

    // TODO: this is an ugly workaround to get beautiful spaciing for tables,
    // because tera whitespace control appears to not be providing what is needed
    // to control the number of spaces in the loop elements properly
    pub space: String,
}

impl ReleasePage {
    pub fn from_data_release(
        data: &data::Releases,
        version: Version,
        previous: Option<(Version, &data::Release)>,
        next: Option<(Version, &data::Release)>,
        end_date: Date,
    ) -> Result<Self> {
        let series_number = SeriesNumber::from(&version);

        let series = data
            .series
            .get(&series_number)
            .context("Couldn't find release series {series_number:?}.")?;

        let release = series
            .releases
            .get(&version)
            .context("Couldn't find release {version:?} in series {series_number}.")?;

        Ok(Self {
            product_name: data.product_name.clone(),
            series: ReleaseSeries::from_data_release_series(
                series_number,
                series,
                &data.components,
            )?,
            release: Release::from_data_release(
                version,
                previous,
                next,
                end_date,
                release,
                &data.components,
            )?,
            space: " ".to_string(),
        })
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmptyReleaseComponent {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Component {
    pub identifier: ComponentIdentifier,
    pub name: ComponentName,
    pub gitlab_url: String,
}

impl Component {
    fn from_data_component(identifier: ComponentIdentifier, component: &data::Component) -> Self {
        Self {
            identifier,
            name: component.name.clone(),
            gitlab_url: component.gitlab_url.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseSeries {
    pub version: SeriesNumber,
    pub codename: SeriesCodename,
    pub end_of_life: Date,
    pub releases: Vec<Release>,
    pub markdown_anchor: String,
}

impl ReleaseSeries {
    fn from_data_release_series(
        version: SeriesNumber,
        release_series: &data::ReleaseSeries,
        components: &BTreeMap<ComponentIdentifier, data::Component>,
    ) -> Result<Self> {
        let markdown_anchor = format!("{} ({})", version, release_series.codename)
            .replace(['(', ')', '.'], "")
            .replace(' ', "-")
            .to_lowercase();
        let releases_with_padding = std::iter::once(None)
            .chain(release_series.releases.iter().map(Some))
            .chain(std::iter::once(None))
            .collect::<Vec<_>>();

        Ok(Self {
            version: version.clone(),
            codename: release_series.codename.clone(),
            end_of_life: release_series.end_of_life,
            releases: releases_with_padding
                .windows(3)
                .map(|window| {
                    let previous = window[0].map(|(v, r)| (v.clone(), r));
                    let (version, release) = window[1].unwrap();
                    let next = window[2].map(|(v, r)| (v.clone(), r));
                    let end_date = next
                        .as_ref()
                        .map(|(v, _)| {
                            release_series
                                .releases
                                .get(v)
                                .unwrap_or_else(|| panic!("version {v} not found"))
                                .date
                        })
                        .unwrap_or(release_series.end_of_life);
                    Release::from_data_release(
                        version.clone(),
                        previous,
                        next,
                        end_date,
                        release,
                        components,
                    )
                })
                .collect::<Result<_, anyhow::Error>>()?,
            markdown_anchor,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Release {
    pub version: Version,
    pub previous: Option<Version>,
    pub next: Option<Version>,
    pub date: Date,
    pub end_date: Date,
    pub components: Vec<ReleaseComponent>,
    pub components_by_identifier: BTreeMap<ComponentIdentifier, ReleaseComponent>,
    pub component_releases: BTreeMap<ComponentIdentifier, Vec<ComponentRelease>>,
}

impl Release {
    fn from_data_release(
        version: Version,
        previous: Option<(Version, &data::Release)>,
        next: Option<(Version, &data::Release)>,
        end_date: Date,
        release: &data::Release,
        components: &BTreeMap<ComponentIdentifier, data::Component>,
    ) -> Result<Self> {
        let mut component_releases = BTreeMap::new();

        for (component_identifier, component_version) in &release.components {
            let previous = previous
                .iter()
                .flat_map(|(_, release)| release.components.get(component_identifier))
                .next();

            if let Some(component) = components.get(component_identifier) {
                let releases = component
                    .get_releases(previous.cloned(), component_version.clone())
                    .into_iter()
                    .map(|(version, release)| {
                        ComponentRelease::from_data_component_release(
                            version,
                            component.gitlab_url.clone(),
                            &release,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseComponent {
    pub identifier: ComponentIdentifier,
    pub version: Version,
    pub gitlab_url: String,
}

impl ReleaseComponent {
    fn from_data_component(
        identifier: ComponentIdentifier,
        version: Version,
        gitlab_url: String,
    ) -> Self {
        Self {
            identifier,
            version,
            gitlab_url,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentRelease {
    pub version: Version,
    pub gitlab_url: String,
    pub changelog: String,
}

impl ComponentRelease {
    fn from_data_component_release(
        version: Version,
        gitlab_url: String,
        component_release: &data::ComponentRelease,
    ) -> Self {
        Self {
            version,
            gitlab_url,
            changelog: component_release.changelog.clone(),
        }
    }
}
