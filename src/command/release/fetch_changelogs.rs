// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::Path;

use anyhow::{bail, Context, Result};
use clap::Args;
use gitlab::{
    api::{projects, Query},
    Gitlab, ReleaseTag,
};
use owo_colors::OwoColorize;
use semver::Version;
use url::Url;

use crate::data::{
    read_release_file, write_releases_file, Component, ComponentIdentifier, ComponentRelease,
    ComponentVersion,
};

#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct FetchChangelogsArgs {
    /// The GitLab access token. This token requires at least `api:read` capabilities.
    #[clap(long, env = "GITLAB_TOKEN")]
    pub gitlab_token: String,
}

impl FetchChangelogsArgs {
    pub fn execute<R: AsRef<Path>>(self, release_file: R, version: &Version) -> Result<()> {
        let mut raw_data = read_release_file(&release_file)?;

        let series_number = version.into();
        let series = raw_data
            .series
            .get(&series_number)
            .with_context(|| format!("Release series {series_number} not found"))?;
        let release = series
            .releases
            .get(version)
            .with_context(|| format!("Release {version} not found in series {series_number}"))?;

        let mut errors = Vec::new();
        let mut overall = 0usize;
        let mut successful = 0usize;
        for (identifier, version) in release.components.clone() {
            overall += 1;
            let component = raw_data
                .components
                .get_mut(&identifier)
                .with_context(|| format!("Couldn't find component {:?}", identifier))?;
            if let Err(e) =
                self.fetch_changelog(identifier, component, &version, &self.gitlab_token)
            {
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

        write_releases_file(&release_file, raw_data)?;

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
        version: &ComponentVersion,
        token: &str,
    ) -> Result<()> {
        print!("Looking up release {version} of {component_identifier}…");

        let Some(gitlab_url) = &component.gitlab_url else {
            println!(" no GitLab URL, {}", "SKIPPING".yellow());
            let component_release = ComponentRelease {
                date: None,
                changelog: None,
            };
            component.insert_release(version.clone(), component_release);

            return Ok(());
        };

        let gitlab_url: Url = gitlab_url.parse()?;
        let host = gitlab_url
            .host_str()
            .with_context(|| format!("No host part found in url {gitlab_url:?}"))?;
        let gitlab = Gitlab::new(host, token)?;

        let project_path = gitlab_url.path().trim_start_matches('/');

        let endpoint = projects::releases::ProjectReleases::builder()
            .project(project_path)
            .build()?;

        let releases: Vec<ReleaseTag> = endpoint.query(&gitlab)?;

        let release_tag = version.prefixed();
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
            component.insert_release(version.clone(), component_release);
        } else {
            println!(" {}", "FAIL".red());
            bail!("Release {version} not found for {component_identifier}");
        }

        Ok(())
    }
}
