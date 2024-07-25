// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};
use time::Date;

use crate::data::ComponentVersion;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseComponentMetadata {
    pub version: ComponentVersion,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<Date>,
}
