// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    PartialOrd,
    Ord,
    Hash,
    derive_more::From,
    derive_more::Into,
    derive_more::Display,
)]
pub struct ComponentCategoryIdentifier(String);
