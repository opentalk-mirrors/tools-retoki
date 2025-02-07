// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context as _, Result};
use time::{format_description::well_known::Rfc3339, Date};

pub fn parse_date(s: &str) -> Result<Date> {
    Date::parse(s, &Rfc3339).with_context(|| format!("Invalid date string {:?}", s))
}
