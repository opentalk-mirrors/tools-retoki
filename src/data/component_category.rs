// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use super::ComponentCategoryName;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentCategory {
    pub name: ComponentCategoryName,
}
