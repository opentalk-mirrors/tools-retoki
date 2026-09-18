// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::Path;

use anyhow::{Context, Result};
use clap::Args;
use gitlab::{
    Gitlab,
    api::{Pagination, Query, paged, projects},
};
use rayon::iter::{IntoParallelIterator as _, ParallelIterator};
use semver::Version;
use tracing::info_span;
use url::Url;

use crate::{
    command::ProfileArgs,
    data::{
        Component, ComponentIdentifier, ComponentRelease, ComponentVersion, read_profile_file,
        read_release_file, write_releases_file,
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

        // Component versions of the preceding release, used to discover versions
        // that were skipped between that release and this one.
        let previous_components = releases
            .previous_release(version)
            .map(|previous| previous.components.clone());

        let release_components: Vec<_> = releases
            .components
            .iter_mut()
            .filter_map(|(ident, comp)| {
                release.components.get(ident).map(|comp_version| {
                    let gitlab_url = profile.component(ident).and_then(|entry| entry.gitlab_url);
                    let previous_version = previous_components
                        .as_ref()
                        .and_then(|components| components.get(ident));
                    (ident, comp_version, comp, gitlab_url, previous_version)
                })
            })
            .collect();

        let res: Vec<_> = release_components
            .into_par_iter()
            .map(
                |(identifier, comp_version, component, gitlab_url, previous_version)| {
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
                        gitlab_url,
                        comp_version,
                        previous_version,
                        &self.gitlab_token,
                    )
                },
            )
            .collect();

        // Sum fetched changelogs and report internal failures
        let successful = res.into_iter().fold(0usize, |mut count, item| {
            match item {
                Ok(fetched) => count += fetched,
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
        gitlab_url: Option<&str>,
        version: &ComponentVersion,
        previous_version: Option<&ComponentVersion>,
        token: &str,
    ) -> Result<usize> {
        tracing::debug!(component = %component_identifier, version = %version, "Looking up release changelog");

        let Some(gitlab_url) = gitlab_url else {
            let component_release = ComponentRelease {
                date: None,
                changelog: None,
            };
            component.insert_release(version.clone(), component_release);
            tracing::info!(component = %component_identifier, "⏭️ No GitLab URL, skipping changelog fetch");
            return Ok(0);
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

        // Fetch every page: GitLab sorts releases by date, not version, so a
        // version in our range could otherwise fall onto an unfetched page.
        let releases: Vec<ReleaseTag> = paged(endpoint, Pagination::All).query(&gitlab)?;

        let selected = select_changelog_tags(&releases, previous_version, version);
        if selected.is_empty() {
            tracing::info!(component = %component_identifier, version = %version, "❌ Changelog not found");
            return Ok(0);
        }

        let mut fetched = 0;
        for (selected_version, description) in selected {
            tracing::info!(component = %component_identifier, version = %selected_version, "✅ Fetched changelog");
            let changelog = if description.is_empty() {
                None
            } else {
                Some(description.to_owned())
            };
            let component_release = ComponentRelease {
                date: None,
                changelog,
            };
            component.insert_release(selected_version, component_release);
            fetched += 1;
        }
        Ok(fetched)
    }
}

/// Strip a leading `v` (if present) and parse the remainder as a semver version.
fn parse_semver_tag(tag_name: &str) -> Option<Version> {
    tag_name.strip_prefix('v').unwrap_or(tag_name).parse().ok()
}

/// Select the release tags whose changelogs should be stored for `current`.
///
/// Always includes an exact match for `current`. When both `current` and
/// `previous` are semver versions, it additionally includes every version
/// strictly between them, so changelogs of skipped intermediate versions are
/// not lost.
fn select_changelog_tags<'a>(
    tags: &'a [ReleaseTag],
    previous: Option<&ComponentVersion>,
    current: &ComponentVersion,
) -> Vec<(ComponentVersion, &'a str)> {
    let mut selected = Vec::new();

    let current_tag = current.prefixed();
    if let Some(tag) = tags.iter().find(|tag| tag.tag_name == current_tag) {
        selected.push((current.clone(), tag.description.as_str()));
    }

    // Enumerating skipped versions is only well-defined when both bounds are
    // semver; otherwise fall back to the exact match added above.
    let enumerate =
        previous.is_some_and(|p| p.as_semver().is_some()) && current.as_semver().is_some();
    if let (true, Some(previous)) = (enumerate, previous) {
        for tag in tags {
            let Some(version) = parse_semver_tag(&tag.tag_name) else {
                continue;
            };
            let version = ComponentVersion::Semver(version);
            if &version > previous && &version < current {
                selected.push((version, tag.description.as_str()));
            }
        }
    }

    selected
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(name: &str, description: &str) -> ReleaseTag {
        ReleaseTag {
            tag_name: name.to_owned(),
            description: description.to_owned(),
        }
    }

    fn semver(version: &str) -> ComponentVersion {
        ComponentVersion::Semver(version.parse().expect("valid semver"))
    }

    #[test]
    fn includes_skipped_intermediate_versions() {
        let tags = vec![
            tag("v1.17.1", "one"),
            tag("v1.17.2", "two"),
            tag("v1.17.3", "three"),
        ];
        let previous = semver("1.17.1");
        let current = semver("1.17.3");

        let selected = select_changelog_tags(&tags, Some(&previous), &current);

        let versions: Vec<_> = selected.iter().map(|(v, _)| v.to_string()).collect();
        assert!(versions.contains(&"1.17.3".to_owned()));
        assert!(versions.contains(&"1.17.2".to_owned()));
        assert!(!versions.contains(&"1.17.1".to_owned()));
        assert_eq!(selected.len(), 2);
    }

    #[test]
    fn includes_intermediate_prereleases() {
        let tags = vec![
            tag("v1.17.2-rc.1", "rc"),
            tag("v1.17.2", "two"),
            tag("v1.17.3", "three"),
        ];
        let previous = semver("1.17.1");
        let current = semver("1.17.3");

        let selected = select_changelog_tags(&tags, Some(&previous), &current);

        let versions: Vec<_> = selected.iter().map(|(v, _)| v.to_string()).collect();
        assert!(versions.contains(&"1.17.2-rc.1".to_owned()));
        assert!(versions.contains(&"1.17.2".to_owned()));
        assert!(versions.contains(&"1.17.3".to_owned()));
    }

    #[test]
    fn without_previous_only_exact_match() {
        let tags = vec![tag("v1.17.2", "two"), tag("v1.17.3", "three")];
        let current = semver("1.17.3");

        let selected = select_changelog_tags(&tags, None, &current);

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].0.to_string(), "1.17.3");
    }

    #[test]
    fn unchanged_component_only_current() {
        let tags = vec![tag("v1.17.2", "two"), tag("v1.17.3", "three")];
        let previous = semver("1.17.3");
        let current = semver("1.17.3");

        let selected = select_changelog_tags(&tags, Some(&previous), &current);

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].0.to_string(), "1.17.3");
    }

    #[test]
    fn non_semver_current_only_exact_match() {
        let tags = vec![tag("nightly", "n"), tag("v1.17.3", "three")];
        let previous = semver("1.17.1");
        let current = ComponentVersion::Other("nightly".to_owned());

        let selected = select_changelog_tags(&tags, Some(&previous), &current);

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].0.to_string(), "nightly");
    }
}
