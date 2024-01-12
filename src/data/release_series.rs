// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use semver::Version;
use serde::{Deserialize, Serialize};
use time::Date;

use super::{Release, SeriesCodename};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseSeries {
    pub codename: Option<SeriesCodename>,
    pub end_of_life: Date,
    pub releases: BTreeMap<Version, Release>,
}
