// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::Path;

use anyhow::{bail, Context, Result};
use clap::Args;
use gitlab::{
    api::{projects, Query},
    Gitlab, ReleaseTag,
};
use indicatif::{MultiProgress, ProgressBar};
use owo_colors::OwoColorize;
use rayon::iter::{IntoParallelIterator as _, ParallelIterator};
use semver::Version;
use url::Url;

use crate::{
    command::ProfileArgs,
    data::{
        read_profile_file, read_release_file, write_releases_file, Component, ComponentIdentifier,
        ComponentProfile, ComponentRelease, ComponentVersion,
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct FetchChangelogsArgs {
    /// The GitLab access token. This token requires at least `api:read` capabilities.
    #[clap(long, env = "GITLAB_TOKEN")]
    pub gitlab_token: String,

    #[clap(flatten)]
    pub profile: ProfileArgs,
}

impl FetchChangelogsArgs {
    pub fn execute<R: AsRef<Path>>(self, release_file: R, version: &Version) -> Result<()> {
        let mut releases = read_release_file(&release_file)?;
        let profile = read_profile_file(
            &release_file,
            &self.profile.profile,
            self.profile.profile_path.as_deref(),
        )?;

        let series_number = version.into();
        let series = releases
            .series
            .get(&series_number)
            .with_context(|| format!("Release series {series_number} not found"))?;
        let release = series
            .releases
            .get(version)
            .with_context(|| format!("Release {version} not found in series {series_number}"))?;

        let multi_bar = MultiProgress::new();
        let release_components: Vec<_> = releases
            .components
            .iter_mut()
            .filter_map(|(ident, comp)| {
                release.components.get(ident).map(|comp_version| {
                    let bar = multi_bar.add(ProgressBar::new_spinner());
                    let comp_profile = profile.components.get(ident);
                    (ident, comp_version, comp, comp_profile, bar)
                })
            })
            .collect();
        multi_bar.set_move_cursor(true);

        let res: Vec<_> = release_components
            .into_par_iter()
            .map(
                |(identifier, comp_version, component, component_profile, bar)| {
                    Self::fetch_changelog(
                        identifier.clone(),
                        component,
                        component_profile,
                        comp_version,
                        &self.gitlab_token,
                        bar,
                    )
                },
            )
            .collect();

        // Collect errors and count successes
        let successful = res.into_iter().fold(0usize, |mut count, item| {
            if let Err(e) = item {
                eprintln!("{}", e.to_string().red());
            } else {
                count += 1;
            }
            count
        });

        write_releases_file(&release_file, releases)?;

        println!();
        println!(
            "Changelogs in {} has been {}",
            release_file.as_ref().to_string_lossy().bold(),
            format!("updated for {successful} projects").green()
        );

        Ok(())
    }

    fn fetch_changelog(
        component_identifier: ComponentIdentifier,
        component: &mut Component,
        component_profile: Option<&ComponentProfile>,
        version: &ComponentVersion,
        token: &str,
        bar: ProgressBar,
    ) -> Result<()> {
        bar.set_message(format!(
            "Looking up release {version} of {component_identifier}…"
        ));
        let Some(gitlab_url) = &component_profile.and_then(|profile| profile.gitlab_url.as_deref())
        else {
            let component_release = ComponentRelease {
                date: None,
                changelog: None,
            };
            component.insert_release(version.clone(), component_release);
            bar.finish_with_message(format!(
                "{component_identifier} has no GitLab URL, {}",
                "SKIPPING".yellow()
            ));
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

        bar.tick();
        let releases: Vec<ReleaseTag> = endpoint.query(&gitlab)?;

        let release_tag = version.prefixed();
        if let Some(ReleaseTag {
            tag_name: _,
            description,
        }) = releases.into_iter().find(|r| r.tag_name == release_tag)
        {
            bar.finish_with_message(format!(
                "{} - Changelog for {component_identifier} v{version} fetched",
                "OK".green()
            ));
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
            bar.finish_with_message(format!(
                "{} - Changelog for {component_identifier} v{version} not found",
                "FAIL".red()
            ));
            bail!("Release {version} not found for {component_identifier}");
        }

        Ok(())
    }
}
