// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{fs::File, path::Path};

use anyhow::{bail, Context, Result};
use clap::Args;
use gitlab::{
    api::{projects, Query},
    Gitlab, ReleaseTag,
};
use owo_colors::OwoColorize;
use semver::Version;
use url::Url;

use crate::{
    command::utils::write_releases_yml_file,
    data::{self, Component, ComponentIdentifier, ComponentRelease},
};

const GITLAB_TOKEN_ENV_VAR: &str = "GITLAB_TOKEN";

#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct FetchChangelogsArgs {}

impl FetchChangelogsArgs {
    pub fn execute<R: AsRef<Path>>(self, release_file: R, version: &Version) -> Result<()> {
        let mut raw_data: data::Releases = {
            let file = File::open(&release_file).context(format!(
                "Couldn't open release file {:?}",
                release_file.as_ref()
            ))?;
            serde_yaml::from_reader(file)?
        };

        let series_number = version.into();
        let series = raw_data
            .series
            .get(&series_number)
            .with_context(|| format!("Release series {series_number} not found"))?;
        let release = series
            .releases
            .get(version)
            .with_context(|| format!("Release {version} not found in series {series_number}"))?;

        let gitlab_token = std::env::var(GITLAB_TOKEN_ENV_VAR).context(format!(
            "Environment varible {GITLAB_TOKEN_ENV_VAR} is not set"
        ))?;

        let mut errors = Vec::new();
        let mut overall = 0usize;
        let mut successful = 0usize;
        for (identifier, version) in release.components.clone() {
            overall += 1;
            let component = raw_data
                .components
                .get_mut(&identifier)
                .context(format!("Couldn't find component {:?}", identifier))?;
            if let Err(e) = self.fetch_changelog(identifier, component, &version, &gitlab_token) {
                errors.push(e);
            } else {
                successful += 1;
            };
        }

        println!();
        println!("Fetched {successful}/{overall} projects successfully");
        if !errors.is_empty() {
            eprintln!();
            for error in errors {
                eprintln!("{}", error.to_string().red());
            }
        }

        write_releases_yml_file(&release_file, raw_data)?;

        println!();
        println!(
            "Changelogs in {} has been {}",
            release_file.as_ref().to_string_lossy().bold(),
            format!("updated for {successful} projects").green()
        );

        Ok(())
    }

    fn fetch_changelog(
        &self,
        component_identifier: ComponentIdentifier,
        component: &mut Component,
        version: &Version,
        token: &str,
    ) -> Result<()> {
        let Some(gitlab_url) = &component.gitlab_url else {
            print!("Component {component_identifier} has no GitLab URL, skipping release lookup…");
            return Ok(());
        };
        print!("Looking up release {version} of {component_identifier}…");

        let gitlab_url: Url = gitlab_url.parse()?;
        let host = gitlab_url
            .host_str()
            .context(format!("No host part found in url {gitlab_url:?}"))?;
        let gitlab = Gitlab::new(host, token)?;

        let project_path = gitlab_url.path().trim_start_matches('/');

        let endpoint = projects::releases::ProjectReleases::builder()
            .project(project_path)
            .build()?;

        let releases: Vec<ReleaseTag> = endpoint.query(&gitlab)?;

        let release_tag = format!("v{version}");
        if let Some(ReleaseTag {
            tag_name: _,
            description,
        }) = releases.into_iter().find(|r| r.tag_name == release_tag)
        {
            println!(" {}", "OK".green());
            let changelog = if description.is_empty() {
                None
            } else {
                Some(description)
            };

            let component_release = ComponentRelease {
                date: None,
                changelog,
            };
            component
                .releases
                .insert(version.clone(), component_release);
            // sort in reverse order, highest version number first
            component
                .releases
                .sort_unstable_by(|k1, _v1, k2, _v2| k2.cmp(k1))
        } else {
            println!(" {}", "FAIL".red());
            bail!("Release {version} not found for {component_identifier}");
        }

        Ok(())
    }
}
