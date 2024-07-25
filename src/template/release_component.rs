// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use crate::data::{ComponentCategoryName, ComponentIdentifier, ComponentVersion};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseComponent {
    pub identifier: ComponentIdentifier,
    pub category: ComponentCategoryName,
    pub version: ComponentVersion,
    pub gitlab_url: Option<String>,
}

impl ReleaseComponent {
    pub fn from_data_component(
        identifier: ComponentIdentifier,
        category: ComponentCategoryName,
        version: ComponentVersion,
        gitlab_url: Option<String>,
    ) -> Self {
        Self {
            identifier,
            category,
            version,
            gitlab_url,
        }
    }
}
