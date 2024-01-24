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

use crate::{
    data::{self, StripReleases},
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

    /// Don't link to GitLab releases.
    #[clap(long)]
    pub without_gitlab_release_links: bool,
}

impl GenerateArgs {
    pub fn execute<R: AsRef<Path>>(self, release_file: R) -> Result<()> {
        let mut tera = Tera::default();
        tera.add_raw_template("README.md", include_str!("../../templates/README.md"))?;
        tera.add_raw_template("release.md", include_str!("../../templates/release.md"))?;
        tera.add_raw_template("component.md", include_str!("../../templates/component.md"))?;

        let file = File::open(&release_file).context(format!(
            "Couldn't open release file {:?}",
            release_file.as_ref()
        ))?;
        let strip_releases = if self.without_prereleases {
            StripReleases::PreReleases
        } else {
            StripReleases::ObsoletePreReleases
        };

        let raw_data: data::Releases = serde_yaml::from_reader::<_, data::Releases>(file)?
            .with_releases_stripped(strip_releases);

        let target_dir = create_and_canonicalize_dir(self.target_dir)?;
        let releases_dir = create_and_canonicalize_dir(target_dir.join("releases"))?;
        let components_dir = create_and_canonicalize_dir(target_dir.join("components"))?;

        std::fs::create_dir_all(&releases_dir)
            .context(format!("Couldn't create releases dir {:?}", releases_dir))?;

        {
            let mut template_data = template::Readme::from_data_releases(&raw_data)?;
            template_data.show_gantt_chart = !self.without_readme_gantt_chart;
            template_data.show_gitlab_release_links = !self.without_gitlab_release_links;
            let rendered = tera.render("README.md", &Context::from_serialize(&template_data)?)?;
            let relative_path = "README.md";
            let full_path = target_dir.join(relative_path);
            println!("Writing file {full_path:?}");
            let mut file = File::create(&full_path)
                .context(format!("Couldn't create file {:?}", full_path))?;
            write!(file, "{}", rendered)?;
        }

        let mut position = 0;
        for series in raw_data.series.values().rev() {
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
                    &raw_data,
                    version.clone(),
                    previous,
                    next,
                    end_date,
                    position,
                )?;
                template_data.show_gitlab_release_links = !self.without_gitlab_release_links;
                let rendered =
                    tera.render("release.md", &Context::from_serialize(&template_data)?)?;
                let relative_path = releases_dir.join(&format!("{version}.md"));
                let full_path = target_dir.join(&relative_path);
                println!("Writing file {full_path:?}");
                let mut file = File::create(&full_path)
                    .context(format!("Couldn't create file {:?}", full_path))?;
                write!(file, "{}", rendered)?;
                position += 1;
            }
        }

        for (position, component) in raw_data.components.iter().enumerate() {
            let template_data = template::ComponentPage::from_data_component(
                component.0,
                component.1,
                &raw_data.product_name,
                &raw_data,
                position,
            );

            let rendered =
                tera.render("component.md", &Context::from_serialize(&template_data)?)?;
            let relative_path = components_dir.join(&format!("{}.md", component.0));
            let full_path = target_dir.join(&relative_path);
            println!("Writing file {full_path:?}");
            let mut file = File::create(&full_path)
                .context(format!("Couldn't create file: {:?}", full_path))?;
            write!(file, "{}", rendered)?;
        }

        Ok(())
    }
}

fn create_and_canonicalize_dir<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let path = path.as_ref();
    std::fs::create_dir_all(path).context(format!("Couldn't create dir {path:?}"))?;
    path.canonicalize()
        .with_context(|| format!("Couldn't canonicalize directory path {path:?}"))
}
