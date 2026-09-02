// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::Path;

use anyhow::{Context, Result};
use clap::Args;
use gitlab::{
    Gitlab,
    api::{Query, projects},
};
use rayon::iter::{IntoParallelIterator as _, ParallelIterator};
use semver::Version;
use tracing::info_span;
use url::Url;

use crate::{
    command::ProfileArgs,
    data::{
        Component, ComponentIdentifier, ComponentProfile, ComponentRelease, ComponentVersion,
        read_profile_file, read_release_file, write_releases_file,
    },
    helper::progress,
};

#[derive(serde::Deserialize)]
pub struct ReleaseTag {
    tag_name: String,
    description: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct FetchChangelogsArgs {
    /// The GitLab access token. This token requires at least `api:read` capabilities.
    #[clap(long, env = "GITLAB_TOKEN")]
    pub gitlab_token: String,

    #[clap(flatten)]
    pub profile_args: ProfileArgs,
}

impl FetchChangelogsArgs {
    pub fn execute<R: AsRef<Path>>(self, release_file: R, version: &Version) -> Result<()> {
        tracing::info!(
            release_file = %release_file.as_ref().display(),
            release = %version,
            "Fetching changelogs"
        );

        let mut releases = read_release_file(&release_file)?;
        let profile = read_profile_file(&self.profile_args.profile)?;

        let series_number = version.into();
        let series = releases
            .series
            .get(&series_number)
            .with_context(|| format!("Release series {series_number} not found"))?;
        let release = series
            .releases
            .get(version)
            .with_context(|| format!("Release {version} not found in series {series_number}"))?;

        let release_components: Vec<_> = releases
            .components
            .iter_mut()
            .filter_map(|(ident, comp)| {
                release.components.get(ident).map(|comp_version| {
                    let comp_profile = profile.components.get(ident);
                    (ident, comp_version, comp, comp_profile)
                })
            })
            .collect();

        let res: Vec<_> = release_components
            .into_par_iter()
            .map(|(identifier, comp_version, component, component_profile)| {
                let task_span = info_span!("fetch_changelog");
                progress::start(
                    &task_span,
                    None,
                    &format!("Fetching changelog for {identifier} {comp_version}"),
                    None,
                );
                let _task_guard = task_span.enter();
                Self::fetch_changelog(
                    identifier.clone(),
                    component,
                    component_profile,
                    comp_version,
                    &self.gitlab_token,
                )
            })
            .collect();

        // Count fetched changelogs and report internal failures
        let successful = res.into_iter().fold(0usize, |mut count, item| {
            match item {
                Ok(true) => count += 1,
                Ok(false) => {}
                Err(e) => tracing::error!(error = %e, "Failed to fetch changelog"),
            }
            count
        });

        write_releases_file(&release_file, releases)?;

        tracing::info!(
            release_file = %release_file.as_ref().display(),
            successful,
            "Changelogs updated"
        );

        Ok(())
    }

    fn fetch_changelog(
        component_identifier: ComponentIdentifier,
        component: &mut Component,
        component_profile: Option<&ComponentProfile>,
        version: &ComponentVersion,
        token: &str,
    ) -> Result<bool> {
        tracing::debug!(component = %component_identifier, version = %version, "Looking up release changelog");

        let Some(gitlab_url) = &component_profile.and_then(|profile| profile.gitlab_url.as_deref())
        else {
            let component_release = ComponentRelease {
                date: None,
                changelog: None,
            };
            component.insert_release(version.clone(), component_release);
            tracing::info!(component = %component_identifier, "⏭️ No GitLab URL, skipping changelog fetch");
            return Ok(false);
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
        if let Some(ReleaseTag { description, .. }) =
            releases.into_iter().find(|r| r.tag_name == release_tag)
        {
            tracing::info!(component = %component_identifier, version = %version, "✅ Fetched changelog");
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
            Ok(true)
        } else {
            tracing::info!(component = %component_identifier, version = %version, "❌ Changelog not found");
            Ok(false)
        }
    }
}
