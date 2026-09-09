// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::path::Path;

use anyhow::Context;
use clap::Args;
use semver::Version;
use tracing::info_span;
use url::Url;

use crate::{
    bot_config::Config,
    data::{ProductTicket, SeriesNumber, read_release_file, write_releases_file},
    gitlab_service::GitlabService,
    helper::progress,
    vcs_service::{IssueFilter, IssueScope, VcsService},
};

const DEFAULT_PRODUCT_REPO_URL: &str = "https://git.opentalk.dev/opentalk/product/tickets";
const RELEASE_LABEL_PREFIX: &str = "release-";

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct FetchProductTicketsArgs {
    #[clap(long, default_value = DEFAULT_PRODUCT_REPO_URL)]
    product_repo: Url,
}

impl FetchProductTicketsArgs {
    pub fn execute<R: AsRef<Path>>(self, release_file: R, version: &Version) -> anyhow::Result<()> {
        tracing::info!(
            release_file = %release_file.as_ref().display(),
            release = %version,
            "Fetching product tickets"
        );

        let progress_span = info_span!("fetch_product_tickets_progress");
        progress::start(
            &progress_span,
            None,
            "Querying product tickets from GitLab",
            Some("Finished querying product tickets"),
        );

        let config = Config::load()?;
        let vcs = GitlabService::connect(
            config.gitlab_url.clone(),
            config.gitlab_token.clone(),
            config.gitlab_group.clone(),
        )?;

        let mut releases = read_release_file(&release_file)?;
        let series_nr = SeriesNumber::from(version);
        let release = releases
            .series
            .get_mut(&series_nr)
            .with_context(|| format!("Release series {series_nr} not found"))?
            .releases
            .get_mut(version)
            .with_context(|| format!("Release {version} not found in series {series_nr}"))?;

        release.tickets = self.fetch_product_tickets(&vcs, version)?;

        tracing::info!(release = %version, tickets = release.tickets.len(), "Fetched product tickets");

        write_releases_file(release_file, releases)
    }

    fn fetch_product_tickets(
        &self,
        vcs: &dyn VcsService,
        version: &Version,
    ) -> anyhow::Result<Vec<ProductTicket>> {
        let project = vcs.project_path_from_url(&self.product_repo)?;
        let label = release_label(version);

        let issues = vcs
            .fetch_issues(
                IssueScope::Project(&project),
                &IssueFilter {
                    labels: &[&label],
                    ..Default::default()
                },
            )
            .context("Failed to fetch tickets")?;

        Ok(issues
            .into_iter()
            .map(|issue| {
                ProductTicket::new(
                    issue.title,
                    issue.iid,
                    issue.web_url.to_string(),
                    issue.labels,
                )
            })
            .collect())
    }
}

fn release_label(version: &Version) -> String {
    format!("{RELEASE_LABEL_PREFIX}{version}")
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::DEFAULT_PRODUCT_REPO_URL;

    #[test]
    fn product_url_is_valid() {
        let url: Url = DEFAULT_PRODUCT_REPO_URL
            .parse()
            .expect("Default product repo URL must be valid");

        assert!(url.host().is_some());
        assert!(!url.path().is_empty());
    }
}
