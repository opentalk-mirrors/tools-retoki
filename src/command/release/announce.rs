// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::Path;

use anyhow::{Context as _, Result};
use clap::Args;
use semver::Version;
use tera::{Context, Tera};
use time::OffsetDateTime;

use crate::{
    command::ProfileArgs,
    data::{SeriesNumber, is_prerelease, read_profile_file, read_release_file},
    template::ReleasePage,
};

mod md_to_text;

/// Render a release announcement as plain text.
///
/// The announcement is rendered as plain text suitable for email and mailing
/// lists that are also published on the web. Hyperlinks are retained inline in
/// angle brackets and Markdown release notes (including tables and footnotes)
/// are converted to plain text.
#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct AnnounceArgs {
    #[clap(flatten)]
    pub profile_args: ProfileArgs,
}

impl AnnounceArgs {
    pub fn execute<R: AsRef<Path>>(self, release_file: R, version: &Version) -> Result<()> {
        let rendered = self.render(release_file, version)?;
        print!("{rendered}");
        Ok(())
    }

    /// Render the announcement for a release version as plain text.
    pub fn render<R: AsRef<Path>>(self, release_file: R, version: &Version) -> Result<String> {
        let releases = read_release_file(&release_file)?;
        let profile =
            read_profile_file(&self.profile_args.profile).context("Failed to read profile")?;

        let series_number = SeriesNumber::from(version);
        let series = releases
            .series
            .get(&series_number)
            .with_context(|| format!("Release series {series_number} not found"))?;

        // The previous release bounds the range of component releases so that
        // only ones not already part of an earlier product release are rendered.
        // Prerelease versions are skipped so the base is the last stable release.
        let previous = series
            .releases
            .get_index_of(version)
            .and_then(|index| series.releases.get_range(..index))
            .into_iter()
            .flatten()
            .rfind(|(previous_version, _)| !is_prerelease(previous_version))
            .map(|(previous_version, previous_release)| {
                (previous_version.clone(), previous_release)
            });

        let template_data = ReleasePage::from_data_release(
            &releases,
            &profile,
            version.clone(),
            previous,
            None,
            series.end_of_life,
            false,
            OffsetDateTime::now_utc().date(),
        )?;

        let template_name = "announcement.txt";
        let template_source = include_str!("../../../templates/announcement.txt");

        let mut tera = Tera::default();
        tera.register_filter("md_to_text", md_to_text::md_to_text_filter);
        tera.add_raw_template(template_name, template_source)?;
        let rendered = tera.render(template_name, &Context::from_serialize(&template_data)?)?;

        Ok(rendered)
    }
}
