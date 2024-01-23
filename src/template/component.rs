// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use crate::data::{self, ComponentIdentifier, ComponentName};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Component {
    pub identifier: ComponentIdentifier,
    pub name: ComponentName,
    pub gitlab_url: Option<String>,
}

impl Component {
    pub fn from_data_component(
        identifier: ComponentIdentifier,
        component: &data::Component,
    ) -> Self {
        Self {
            identifier,
            name: component.name.clone(),
            gitlab_url: component.gitlab_url.clone(),
        }
    }
}
