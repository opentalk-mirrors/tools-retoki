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
    /// Generate the release information from a `releases.json` file.
    Generate {
        /// The JSON file containing the structured release information.
        #[clap(long, default_value = "releases.json")]
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

fn generate<R: AsRef<Path>, T: AsRef<Path>>(release_file: R, target_dir: T) -> Result<()> {
    let mut tera = Tera::default();
    tera.add_raw_template("README.md", include_str!("../templates/README.md"))?;
    tera.add_raw_template("release.md", include_str!("../templates/release.md"))?;

    let file = File::open(&release_file).context(format!(
        "Couldn't open release file {:?}",
        release_file.as_ref()
    ))?;
    let raw_data: data::Releases = serde_json::from_reader(file)?;

    let target_dir = target_dir.as_ref().canonicalize()?;
    let releases_dir = target_dir.join("releases").canonicalize()?;
    std::fs::create_dir_all(&releases_dir)
        .context(format!("Couldn't create releases dir {:?}", releases_dir))?;

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

    for serie in raw_data.series.values() {
        for version in serie.releases.keys() {
            let template_data =
                template::ReleasePage::from_data_release(&raw_data, version.clone())?;
            let rendered = tera.render("release.md", &Context::from_serialize(&template_data)?)?;
            let relative_path = releases_dir.join(&format!("{version}.md"));
            let full_path = target_dir.join(&relative_path);
            println!("Writing file {full_path:?}");
            let mut file = File::create(&full_path)
                .context(format!("Couldn't create file {:?}", full_path))?;
            write!(file, "{}", rendered)?;
        }
    }

    Ok(())
}
