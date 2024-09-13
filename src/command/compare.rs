// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use anyhow::Result;
use clap::Args;
use semver::Version;

use crate::data::read_release_file;

#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct CompareArgs {
    /// The `releases.yml` file to which the base `releases.yml` file should be compared.
    /// Pass `-` here to read from stdin.
    target_file: PathBuf,
}

impl CompareArgs {
    pub fn execute<R: AsRef<Path>>(self, release_file: R) -> Result<()> {
        let CompareArgs { target_file } = self;

        let current_data = read_release_file(release_file, Default::default())?;

        let other_data = read_release_file(target_file, Default::default())?;

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
