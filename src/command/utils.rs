// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::fmt::Display;

use anyhow::{Context as _, Result};
use time::{format_description::well_known::Rfc3339, Date};

pub fn parse_date(s: &str) -> Result<Date> {
    Date::parse(s, &Rfc3339).context(format!("Invalid date string {:?}", s))
}

pub fn tabled_display_option<D: Display>(o: &Option<D>) -> String {
    o.as_ref()
        .map(|d| format!("{}", d))
        .unwrap_or_else(|| "-".to_string())
}
