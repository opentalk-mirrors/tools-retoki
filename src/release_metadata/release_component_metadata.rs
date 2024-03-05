// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use semver::Version;
use serde::{Deserialize, Serialize};
use time::Date;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseComponentMetadata {
    pub version: Version,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<Date>,
}
