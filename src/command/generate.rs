// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context as _, Result};
use clap::Args;
use semver::Version;
use tera::{Context, Tera};
use time::{Date, OffsetDateTime};

use super::ProfileArgs;
use crate::{
    command::utils::parse_date,
    data::{
        Component, ComponentIdentifier, Profile, Release, ReleaseFileReadOptions, ReleaseSeries,
        Releases, SeriesNumber, StripReleases, read_profile_file, read_release_file_with_options,
    },
    release_metadata::ReleaseMetadata,
    template::{self, ReleaseSeriesPage},
};

#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct GenerateArgs {
    /// The target directory for the release information.
    #[clap(long, default_value = ".")]
    pub target_dir: PathBuf,

    /// Don't render any prereleases.
    #[clap(long)]
    pub without_prereleases: bool,

    /// Don't render the gantt chart in the README file.
    #[clap(long)]
    pub without_readme_gantt_chart: bool,

    /// Don't render the end-of-life dates for release series in the README file.
    #[clap(long)]
    pub without_readme_end_of_life: bool,

    /// Don't link to GitLab releases.
    #[clap(long)]
    pub without_gitlab_release_links: bool,

    /// Write metadata files in JSON format for each release.
    #[clap(long)]
    pub with_release_metadata_files: bool,

    /// Insert a markdown header with `title` fields
    #[clap(long)]
    pub with_md_header: bool,

    #[clap(flatten)]
    pub profile: ProfileArgs,

    /// The date that is used as the basis for calculating if releases are EOL
    #[clap(long, value_parser = parse_date)]
    pub date: Option<Date>,

    /// The relative documentation base path pointing to where all versioned
    /// documentation is placed relative to the root of the generated release
    /// documentation, e.g. `../`. This is used for inserting relative links to
    /// the documentation into the release information.
    #[clap(long)]
    pub with_relative_documentation_base_path: Option<String>,
}

impl GenerateArgs {
    pub fn execute<R: AsRef<Path>>(self, release_file: R) -> Result<()> {
        let mut tera = Tera::default();
        tera.add_raw_template("README.md", include_str!("../../templates/README.md"))?;
        tera.add_raw_template(
            "navigation.md",
            include_str!("../../templates/navigation.md"),
        )?;
        tera.add_raw_template("release.md", include_str!("../../templates/release.md"))?;
        tera.add_raw_template(
            "release_series.md",
            include_str!("../../templates/release_series.md"),
        )?;
        tera.add_raw_template("component.md", include_str!("../../templates/component.md"))?;

        let date = self
            .date
            .unwrap_or_else(|| OffsetDateTime::now_utc().date());
        let strip_releases = if self.without_prereleases {
            StripReleases::PreReleases
        } else {
            StripReleases::ObsoletePreReleases
        };

        let releases = read_release_file_with_options(
            &release_file,
            ReleaseFileReadOptions {
                strip_prereleases: Some(strip_releases),
            },
        )
        .context("Failed to read release configuration")?;
        let profile = read_profile_file(
            &release_file,
            &self.profile.profile,
            self.profile.profile_path.as_deref(),
        )
        .context("Failed to read profile")?;

        self.render_readme_md(&tera, &releases, &profile, date)?;
        self.render_navigation_md(&tera, &releases, &profile, date)?;

        for (number, series) in releases.series.iter().rev() {
            let versions_with_padding = std::iter::once(None)
                .chain(series.releases.iter().map(Some))
                .chain(std::iter::once(None))
                .collect::<Vec<_>>();

            {
                self.render_release_series_readme_md(
                    &tera, &releases, &profile, number, series, date,
                )?;
            }

            for entry in versions_with_padding.windows(3).rev() {
                let previous = entry[0].map(|(v, r)| (v.clone(), r));
                let version = entry[1].unwrap().0;
                let next = entry[2].map(|(v, r)| (v.clone(), r));

                self.render_release_readme_md(
                    &tera,
                    &releases,
                    &profile,
                    series,
                    VersionWithNeighbors {
                        previous,
                        version,
                        next,
                    },
                    date,
                )?;
            }
        }

        for (identifier, component) in releases.components.iter() {
            self.render_component_md(&tera, &releases, &profile, identifier, component)?;
        }

        Ok(())
    }

    fn render_readme_md(
        &self,
        tera: &Tera,
        releases: &Releases,
        profile: &Profile,
        date: Date,
    ) -> Result<()> {
        let mut template_data = template::Readme::from_data_releases(releases, profile, date)?;
        template_data.show_gantt_chart = !self.without_readme_gantt_chart;
        template_data.show_series_end_of_life = !self.without_readme_end_of_life;
        template_data.show_gitlab_release_links = !self.without_gitlab_release_links;
        template_data.show_md_header = self.with_md_header;
        template_data.relative_documentation_base_path =
            self.with_relative_documentation_base_path.clone();
        let rendered = tera.render("README.md", &Context::from_serialize(&template_data)?)?;
        let target_dir = create_and_canonicalize_dir(&self.target_dir)?;
        let full_path = target_dir.join("README.md");
        println!("Writing file {full_path:?}");
        let mut file = File::create(&full_path)
            .with_context(|| format!("Couldn't create file {full_path:?}"))?;
        write!(file, "{rendered}")?;
        Ok(())
    }

    fn render_navigation_md(
        &self,
        tera: &Tera,
        releases: &Releases,
        profile: &Profile,
        date: Date,
    ) -> Result<()> {
        let mut template_data = template::Readme::from_data_releases(releases, profile, date)?;
        template_data.show_gantt_chart = !self.without_readme_gantt_chart;
        template_data.show_series_end_of_life = !self.without_readme_end_of_life;
        template_data.show_gitlab_release_links = !self.without_gitlab_release_links;
        template_data.show_md_header = self.with_md_header;
        let rendered = tera.render("navigation.md", &Context::from_serialize(&template_data)?)?;
        let target_dir = create_and_canonicalize_dir(&self.target_dir)?;
        let full_path = target_dir.join("navigation.md");
        println!("Writing file {full_path:?}");
        let mut file = File::create(&full_path)
            .with_context(|| format!("Couldn't create file {full_path:?}"))?;
        write!(file, "{rendered}")?;
        Ok(())
    }

    fn render_release_series_readme_md(
        &self,
        tera: &Tera,
        releases: &Releases,
        profile: &Profile,
        series_number: &SeriesNumber,
        series: &ReleaseSeries,
        date: Date,
    ) -> Result<()> {
        let mut template_data = ReleaseSeriesPage::from_data_release_series(
            series_number.clone(),
            releases.product_name.clone(),
            series,
            &releases.components,
            &profile.components,
            &releases.component_categories,
            date,
        )?;
        template_data.show_md_header = self.with_md_header;
        template_data.show_series_end_of_life = !self.without_readme_end_of_life;
        template_data.relative_documentation_base_path =
            self.with_relative_documentation_base_path.clone();

        let target_dir = create_and_canonicalize_dir(&self.target_dir)?;
        let release_series_dir = target_dir.join(format!("{series_number}"));
        let rendered = tera.render(
            "release_series.md",
            &Context::from_serialize(&template_data)?,
        )?;

        std::fs::create_dir_all(target_dir.join(&release_series_dir))?;
        let full_path = release_series_dir.join("README.md");
        println!("Writing file {full_path:?}");
        let mut file = File::create(&full_path)
            .with_context(|| format!("Couldn't create file {full_path:?}"))?;
        write!(file, "{rendered}")?;
        Ok(())
    }

    fn render_release_readme_md(
        &self,
        tera: &Tera,
        releases: &Releases,
        profile: &Profile,
        series: &ReleaseSeries,
        VersionWithNeighbors {
            previous,
            version,
            next,
        }: VersionWithNeighbors<'_>,
        date: Date,
    ) -> Result<()> {
        let end_date = next
            .as_ref()
            .map(|(_, release)| release.date)
            .unwrap_or(series.end_of_life);

        let mut template_data = template::ReleasePage::from_data_release(
            releases,
            profile,
            version.clone(),
            previous,
            next,
            end_date,
            self.with_md_header,
            date,
        )?;
        template_data.show_gitlab_release_links = !self.without_gitlab_release_links;
        let rendered = tera.render("release.md", &Context::from_serialize(&template_data)?)?;
        let target_dir = create_and_canonicalize_dir(&self.target_dir)?;
        let release_dir = target_dir.join(format!("{version}"));
        std::fs::create_dir_all(target_dir.join(&release_dir))?;

        {
            let full_path = release_dir.join("README.md");
            println!("Writing file {full_path:?}");
            let mut file = File::create(&full_path)
                .with_context(|| format!("Couldn't create file {full_path:?}"))?;
            write!(file, "{rendered}")?;
        }

        if self.with_release_metadata_files {
            let release_metadata = ReleaseMetadata::from_data_release(releases, version.clone())?;
            let full_path = release_dir.join("metadata.json");
            println!("Writing file {full_path:?}");
            let file = File::create(&full_path)
                .with_context(|| format!("Couldn't create file {full_path:?}"))?;
            serde_json::to_writer_pretty(file, &release_metadata)?;
        }

        Ok(())
    }

    fn render_component_md(
        &self,
        tera: &Tera,
        releases: &Releases,
        profile: &Profile,
        identifier: &ComponentIdentifier,
        component: &Component,
    ) -> Result<()> {
        let component_profile = profile.components.get(identifier).with_context(|| {
            format!(
                "Missing `{}` component in profile `{}`",
                identifier, profile.profile_name
            )
        })?;
        let template_data = template::ComponentPage::from_data_component(
            identifier,
            component,
            component_profile,
            &releases.product_name,
            releases,
            self.with_md_header,
        );

        let rendered = tera.render("component.md", &Context::from_serialize(&template_data)?)?;
        let target_dir = create_and_canonicalize_dir(&self.target_dir)?;
        let components_dir = create_and_canonicalize_dir(target_dir.join("components"))?;
        let relative_path = components_dir.join(format!("{}.md", identifier));
        let full_path = target_dir.join(&relative_path);
        println!("Writing file {full_path:?}");
        let mut file = File::create(&full_path)
            .with_context(|| format!("Couldn't create file: {full_path:?}"))?;
        write!(file, "{rendered}")?;
        Ok(())
    }
}

struct VersionWithNeighbors<'a> {
    previous: Option<(Version, &'a Release)>,
    version: &'a Version,
    next: Option<(Version, &'a Release)>,
}

fn create_and_canonicalize_dir<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let path = path.as_ref();
    std::fs::create_dir_all(path).with_context(|| format!("Couldn't create dir {path:?}"))?;
    path.canonicalize()
        .with_context(|| format!("Couldn't canonicalize directory path {path:?}"))
}
