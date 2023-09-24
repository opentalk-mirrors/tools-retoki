// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use semver::Version;
use serde::{Deserialize, Serialize};
use time::Date;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Releases {
    pub product_name: String,
    pub components: BTreeMap<String, Component>,
    pub series: BTreeMap<String, ReleaseSeries>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub gitlab_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseSeries {
    pub codename: String,
    pub releases: BTreeMap<Version, Release>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Release {
    pub date: Date,
    pub components: BTreeMap<String, Version>,
}
