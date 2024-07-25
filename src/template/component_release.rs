// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use semver::Version;
use serde::{Deserialize, Serialize};

use crate::data::{self, ComponentVersion};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentRelease {
    pub version: ComponentVersion,
    pub prefixed_version: String,
    pub gitlab_url: Option<String>,
    pub changelog: Option<String>,
    pub product_versions: BTreeSet<Version>,
}

impl ComponentRelease {
    pub fn from_data_component_release(
        version: &ComponentVersion,
        gitlab_url: Option<String>,
        component_release: &data::ComponentRelease,
        product_versions: BTreeSet<Version>,
    ) -> Self {
        Self {
            prefixed_version: version.prefixed(),
            version: version.clone(),
            gitlab_url,
            changelog: component_release.changelog.clone(),
            product_versions,
        }
    }
}
