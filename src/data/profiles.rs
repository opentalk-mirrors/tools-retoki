// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::ComponentIdentifier;

/// A profile allows to specify additional, context dependent information for
/// [`crate::data::Releases`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub profile_name: String,

    pub components: IndexMap<ComponentIdentifier, ComponentProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gitlab_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_base_url: Option<String>,
}
