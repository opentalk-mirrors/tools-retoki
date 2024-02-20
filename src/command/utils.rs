// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{fmt::Display, fs::File, io::BufWriter, path::Path};

use anyhow::{Context as _, Result};
use time::{format_description::well_known::Rfc3339, Date};

use crate::data::Releases;

pub fn parse_date(s: &str) -> Result<Date> {
    Date::parse(s, &Rfc3339).context(format!("Invalid date string {:?}", s))
}

pub fn tabled_display_option<D: Display>(o: &Option<D>) -> String {
    o.as_ref()
        .map(|d| format!("{}", d))
        .unwrap_or_else(|| "-".to_string())
}

pub fn write_releases_yml_file(release_file: impl AsRef<Path>, data: Releases) -> Result<()> {
    let file = File::create(&release_file).context(format!(
        "Couldn't write to release file {:?}",
        release_file.as_ref()
    ))?;
    let writer = BufWriter::new(file);

    serde_yaml::to_writer(writer, &data)
        .context("Couldn't write releases file {release_file:?}")?;

    Ok(())
}
