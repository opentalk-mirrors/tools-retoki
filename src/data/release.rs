// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use time::Date;

use super::{ComponentIdentifier, ComponentVersion};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Release {
    pub date: Date,
    pub components: IndexMap<ComponentIdentifier, ComponentVersion>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_notes: Option<String>,
}
