// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::BTreeSet,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{Context as _, Result};
use clap::Args;
use semver::Version;

use crate::data;

#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct CompareArgs {
    /// The `releases.yml` file to which the base `releases.yml` file should be compared.
    /// Pass `-` here to read from stdin.
    target_file: PathBuf,
}

impl CompareArgs {
    pub fn execute<R: AsRef<Path>>(self, release_file: R) -> Result<()> {
        let CompareArgs { target_file } = self;

        let current_file = File::open(&release_file).context(format!(
            "Couldn't open release file {:?}",
            release_file.as_ref()
        ))?;
        let current_data: data::Releases =
            serde_yaml::from_reader::<_, data::Releases>(current_file)?;

        let other_reader: Box<dyn std::io::Read> = if target_file.as_os_str() == "-" {
            Box::new(std::io::stdin().lock())
        } else {
            Box::new(
                File::open(&target_file)
                    .context(format!("Couldn't open release file {:?}", target_file))?,
            )
        };
        let other_data: data::Releases =
            serde_yaml::from_reader::<_, data::Releases>(other_reader)?;

        let current_versions: BTreeSet<Version> = current_data.all_product_versions();
        let other_versions: BTreeSet<Version> = other_data.all_product_versions();

        for version in current_versions.union(&other_versions) {
            let current = current_data.get_release(version);
            let other = other_data.get_release(version);

            match (current, other) {
                (None, Some(_)) => {
                    println!("- {version}");
                }
                (Some(_), None) => {
                    println!("+ {version}");
                }
                (Some(current), Some(other)) if current == other => {
                    println!("= {version}");
                }
                (Some(_), Some(_)) => {
                    println!("~ {version}")
                }
                (None, None) => {
                    // Should never happen
                }
            }
        }

        Ok(())
    }
}
