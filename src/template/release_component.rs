// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use semver::Version;
use serde::{Deserialize, Serialize};

use crate::data::ComponentIdentifier;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseComponent {
    pub identifier: ComponentIdentifier,
    pub version: Version,
    pub gitlab_url: String,
}

impl ReleaseComponent {
    pub fn from_data_component(
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
