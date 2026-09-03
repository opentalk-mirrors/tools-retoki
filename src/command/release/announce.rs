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
    data::{SeriesNumber, read_profile_file, read_release_file},
    template::ReleasePage,
};

/// Render a release announcement in Markdown.
///
/// The announcement is rendered as Markdown, suitable for Matrix, Element and
/// other chat channels. To convert it to plain text for email or mailing lists,
/// pipe the output through a converter such as pandoc:
///
/// ```sh
/// retoki release <version> announce | pandoc -f markdown -t plain
/// ```
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

    /// Render the announcement for a release version as Markdown.
    pub fn render<R: AsRef<Path>>(self, release_file: R, version: &Version) -> Result<String> {
        let releases = read_release_file(&release_file)?;
        let profile =
            read_profile_file(&self.profile_args.profile).context("Failed to read profile")?;

        let series_number = SeriesNumber::from(version);
        let series = releases
            .series
            .get(&series_number)
            .with_context(|| format!("Release series {series_number} not found"))?;

        let template_data = ReleasePage::from_data_release(
            &releases,
            &profile,
            version.clone(),
            None,
            None,
            series.end_of_life,
            false,
            OffsetDateTime::now_utc().date(),
        )?;

        let template_name = "announcement.md";
        let template_source = include_str!("../../../templates/announcement.md");

        let mut tera = Tera::default();
        tera.add_raw_template(template_name, template_source)?;
        let rendered = tera.render(template_name, &Context::from_serialize(&template_data)?)?;

        Ok(rendered)
    }
}
