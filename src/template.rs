use semver::Version;
use serde::{Deserialize, Serialize};
use time::Date;

use crate::data;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Releases {
    pub product_name: String,
    pub series: Vec<ReleaseSeries>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseSeries {
    pub version: String,
    pub codename: String,
    pub releases: Vec<Release>,
}

impl ReleaseSeries {
    fn from_data_release_series(version: String, release_series: &data::ReleaseSeries) -> Self {
        Self {
            version: version.clone(),
            codename: release_series.codename.clone(),
            releases: release_series
                .releases
                .iter()
                .rev()
                .map(|(version, release)| Release::from_data_release(version.clone(), release))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Release {
    pub version: Version,
    pub date: Date,
    pub components: Vec<Component>,
}

impl Release {
    fn from_data_release(version: Version, release: &data::Release) -> Self {
        Self {
            version,
            date: release.date,
            components: release
                .components
                .iter()
                .map(|(name, version)| {
                    Component::from_data_component(name.clone(), version.clone())
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub version: Version,
}

impl Component {
    fn from_data_component(name: String, version: Version) -> Self {
        Self { name, version }
    }
}

impl From<&data::Releases> for Releases {
    fn from(value: &data::Releases) -> Self {
        Self {
            product_name: value.product_name.clone(),
            series: value
                .series
                .iter()
                .rev()
                .map(|(version, series)| {
                    ReleaseSeries::from_data_release_series(version.clone(), series)
                })
                .collect(),
        }
    }
}
