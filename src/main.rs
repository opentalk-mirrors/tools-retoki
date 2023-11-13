// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context as _, Result};
use clap::{Parser, Subcommand};
use tera::{Context, Tera};

mod data;
mod template;

#[derive(Clone, Debug, PartialEq, Eq, Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, PartialEq, Eq, Subcommand)]
enum Command {
    /// Generate the release information from a `releases.yml` file.
    Generate {
        /// The YAML file containing the structured release information.
        #[clap(long, default_value = "releases.yml")]
        release_file: PathBuf,

        /// The target directory for the release information.
        #[clap(long, default_value = ".")]
        target_dir: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Generate {
            release_file,
            target_dir,
        } => generate(release_file, target_dir),
    }
}

fn create_and_canonicalize_dir<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let path = path.as_ref();
    std::fs::create_dir_all(path).context(format!("Couldn't create dir {path:?}"))?;
    path.canonicalize()
        .with_context(|| format!("Couldn't canonicalize directory path {path:?}"))
}

fn generate<R: AsRef<Path>, T: AsRef<Path>>(release_file: R, target_dir: T) -> Result<()> {
    let mut tera = Tera::default();
    tera.add_raw_template("README.md", include_str!("../templates/README.md"))?;
    tera.add_raw_template("release.md", include_str!("../templates/release.md"))?;
    tera.add_raw_template("component.md", include_str!("../templates/component.md"))?;

    let file = File::open(&release_file).context(format!(
        "Couldn't open release file {:?}",
        release_file.as_ref()
    ))?;
    let raw_data: data::Releases = serde_yaml::from_reader(file)?;

    let target_dir = create_and_canonicalize_dir(target_dir.as_ref())?;
    let releases_dir = create_and_canonicalize_dir(target_dir.join("releases"))?;
    let components_dir = create_and_canonicalize_dir(target_dir.join("components"))?;

    {
        let template_data = template::Readme::from_data_releases(&raw_data)?;
        let rendered = tera.render("README.md", &Context::from_serialize(&template_data)?)?;
        let relative_path = "README.md";
        let full_path = target_dir.join(relative_path);
        println!("Writing file {full_path:?}");
        let mut file =
            File::create(&full_path).context(format!("Couldn't create file {:?}", full_path))?;
        write!(file, "{}", rendered)?;
    }

    for series in raw_data.series.values() {
        let versions_with_padding = std::iter::once(None)
            .chain(series.releases.iter().map(Some))
            .chain(std::iter::once(None))
            .collect::<Vec<_>>();

        for entry in versions_with_padding.windows(3) {
            let previous = entry[0].map(|(v, r)| (v.clone(), r));
            let version = entry[1].unwrap().0;
            let next = entry[2].map(|(v, r)| (v.clone(), r));

            let end_date = next
                .as_ref()
                .map(|(_, release)| release.date)
                .unwrap_or(series.end_of_life);

            let template_data = template::ReleasePage::from_data_release(
                &raw_data,
                version.clone(),
                previous,
                next,
                end_date,
            )?;
            let rendered = tera.render("release.md", &Context::from_serialize(&template_data)?)?;
            let relative_path = releases_dir.join(&format!("{version}.md"));
            let full_path = target_dir.join(&relative_path);
            println!("Writing file {full_path:?}");
            let mut file = File::create(&full_path)
                .context(format!("Couldn't create file {:?}", full_path))?;
            write!(file, "{}", rendered)?;
        }
    }

    for component in &raw_data.components {
        let template_data = template::ComponentPage::from_data_component(
            component.0,
            component.1,
            &raw_data.product_name,
            &raw_data,
        )?;

        let rendered = tera.render("component.md", &Context::from_serialize(&template_data)?)?;
        let relative_path = components_dir.join(&format!("{}.md", component.0));
        let full_path = target_dir.join(&relative_path);
        println!("Writing file {full_path:?}");
        let mut file =
            File::create(&full_path).context(format!("Couldn't create file: {:?}", full_path))?;
        write!(file, "{}", rendered)?;
    }

    Ok(())
}
