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
    Generate { release_file: PathBuf },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Generate { release_file } => generate(release_file),
    }
}

fn generate<R: AsRef<Path>>(release_file: R) -> Result<()> {
    let tera = Tera::new("templates/**/*.md").context("parsing error")?;

    let file = File::open(release_file)?;
    let raw_data: data::Releases = serde_json::from_reader(file)?;
    let template_data = template::Releases::from(&raw_data);

    let readme = tera.render("README.md", &Context::from_serialize(&template_data)?)?;

    {
        let mut file = File::create("README.md")?;
        println!("Writing file README.md");
        write!(file, "{}", readme)?;
    }

    Ok(())
}
