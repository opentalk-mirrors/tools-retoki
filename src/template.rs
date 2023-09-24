// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use anyhow::{Context as _, Result};
use semver::Version;
use serde::{Deserialize, Serialize};
use time::Date;

use crate::data;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Readme {
    pub product_name: String,
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
                .rev()
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
    pub product_name: String,

    #[serde(flatten)]
    pub release: Release,

    pub series: ReleaseSeries,

    // TODO: this is an ugly workaround to get beautiful spaciing for tables,
    // because tera whitespace control appears to not be providing what is needed
    // to control the number of spaces in the loop elements properly
    pub space: String,
}

impl ReleasePage {
    pub fn from_data_release(data: &data::Releases, version: Version) -> Result<Self> {
        let series_number = format!("{}.{}", version.major, version.minor);

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
            release: Release::from_data_release(version, release, &data.components)?,
            space: " ".to_string(),
        })
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmptyReleaseComponent {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Component {
    pub identifier: String,
    pub name: String,
    pub gitlab_url: String,
}

impl Component {
    fn from_data_component(identifier: String, component: &data::Component) -> Self {
        Self {
            identifier,
            name: component.name.clone(),
            gitlab_url: component.gitlab_url.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseSeries {
    pub version: String,
    pub codename: String,
    pub releases: Vec<Release>,
    pub markdown_anchor: String,
}

impl ReleaseSeries {
    fn from_data_release_series(
        version: String,
        release_series: &data::ReleaseSeries,
        components: &BTreeMap<String, data::Component>,
    ) -> Result<Self> {
        let markdown_anchor = format!("{} ({})", version, release_series.codename)
            .replace(['(', ')', '.'], "")
            .replace(' ', "-")
            .to_lowercase();
        Ok(Self {
            version: version.clone(),
            codename: release_series.codename.clone(),
            releases: release_series
                .releases
                .iter()
                .rev()
                .map(|(version, release)| {
                    Release::from_data_release(version.clone(), release, components)
                })
                .collect::<Result<_, anyhow::Error>>()?,
            markdown_anchor,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Release {
    pub version: Version,
    pub date: Date,
    pub components: Vec<ReleaseComponent>,
    pub components_by_identifier: BTreeMap<String, ReleaseComponent>,
}

impl Release {
    fn from_data_release(
        version: Version,
        release: &data::Release,
        components: &BTreeMap<String, data::Component>,
    ) -> Result<Self> {
        Ok(Self {
            version,
            date: release.date,
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
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseComponent {
    pub identifier: String,
    pub version: Version,
    pub gitlab_url: String,
}

impl ReleaseComponent {
    fn from_data_component(identifier: String, version: Version, gitlab_url: String) -> Self {
        Self {
            identifier,
            version,
            gitlab_url,
        }
    }
}
