// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context as _, Result};
use clap::Args;
use tera::{Context, Tera};

use super::ProfileArgs;
use crate::{
    data::{
        ReleaseFileReadOptions, StripReleases, read_profile_file, read_release_file_with_options,
    },
    release_metadata::ReleaseMetadata,
    template,
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

    /// Insert a markdown header with `sidebar_position` and `title` fields
    #[clap(long)]
    pub with_md_header: bool,

    #[clap(flatten)]
    pub profile: ProfileArgs,
}

impl GenerateArgs {
    pub fn execute<R: AsRef<Path>>(self, release_file: R) -> Result<()> {
        let mut tera = Tera::default();
        tera.add_raw_template("README.md", include_str!("../../templates/README.md"))?;
        tera.add_raw_template("release.md", include_str!("../../templates/release.md"))?;
        tera.add_raw_template("component.md", include_str!("../../templates/component.md"))?;

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

        let target_dir = create_and_canonicalize_dir(self.target_dir)?;
        let components_dir = create_and_canonicalize_dir(target_dir.join("components"))?;

        {
            let mut template_data = template::Readme::from_data_releases(&releases, &profile)?;
            template_data.show_gantt_chart = !self.without_readme_gantt_chart;
            template_data.show_series_end_of_life = !self.without_readme_end_of_life;
            template_data.show_gitlab_release_links = !self.without_gitlab_release_links;
            template_data.show_md_header = self.with_md_header;
            let rendered = tera.render("README.md", &Context::from_serialize(&template_data)?)?;
            let relative_path = "README.md";
            let full_path = target_dir.join(relative_path);
            println!("Writing file {full_path:?}");
            let mut file = File::create(&full_path)
                .with_context(|| format!("Couldn't create file {full_path:?}"))?;
            write!(file, "{rendered}")?;
        }

        let mut position = 0;
        for series in releases.series.values().rev() {
            let versions_with_padding = std::iter::once(None)
                .chain(series.releases.iter().map(Some))
                .chain(std::iter::once(None))
                .collect::<Vec<_>>();

            for entry in versions_with_padding.windows(3).rev() {
                let previous = entry[0].map(|(v, r)| (v.clone(), r));
                let version = entry[1].unwrap().0;
                let next = entry[2].map(|(v, r)| (v.clone(), r));

                let end_date = next
                    .as_ref()
                    .map(|(_, release)| release.date)
                    .unwrap_or(series.end_of_life);

                let mut template_data = template::ReleasePage::from_data_release(
                    &releases,
                    &profile,
                    version.clone(),
                    previous,
                    next,
                    end_date,
                    position,
                    self.with_md_header,
                )?;
                template_data.show_gitlab_release_links = !self.without_gitlab_release_links;
                let rendered =
                    tera.render("release.md", &Context::from_serialize(&template_data)?)?;
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
                    let release_metadata =
                        ReleaseMetadata::from_data_release(&releases, version.clone())?;
                    let full_path = release_dir.join("metadata.json");
                    println!("Writing file {full_path:?}");
                    let file = File::create(&full_path)
                        .with_context(|| format!("Couldn't create file {full_path:?}"))?;
                    serde_json::to_writer_pretty(file, &release_metadata)?;
                }

                position += 1;
            }
        }

        for (position, component) in releases.components.iter().enumerate() {
            let component_profile = profile.components.get(component.0).with_context(|| {
                format!(
                    "Missing `{}` component in profile `{}`",
                    component.0, profile.profile_name
                )
            })?;
            let template_data = template::ComponentPage::from_data_component(
                component.0,
                component.1,
                component_profile,
                &releases.product_name,
                &releases,
                position,
                self.with_md_header,
            );

            let rendered =
                tera.render("component.md", &Context::from_serialize(&template_data)?)?;
            let relative_path = components_dir.join(format!("{}.md", component.0));
            let full_path = target_dir.join(&relative_path);
            println!("Writing file {full_path:?}");
            let mut file = File::create(&full_path)
                .with_context(|| format!("Couldn't create file: {full_path:?}"))?;
            write!(file, "{rendered}")?;
        }

        Ok(())
    }
}

fn create_and_canonicalize_dir<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let path = path.as_ref();
    std::fs::create_dir_all(path).with_context(|| format!("Couldn't create dir {path:?}"))?;
    path.canonicalize()
        .with_context(|| format!("Couldn't canonicalize directory path {path:?}"))
}
