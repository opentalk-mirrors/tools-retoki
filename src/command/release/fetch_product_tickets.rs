// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::path::Path;

use anyhow::Context;
use clap::Args;
use gitlab::{
    Gitlab,
    api::{Query, projects},
};
use semver::Version;
use url::Url;

use crate::data::{ProductTicket, SeriesNumber, read_release_file, write_releases_file};

const DEFAULT_PRODUCT_REPO_URL: &str = "https://git.opentalk.dev/opentalk/product/tickets";
const RELEASE_LABEL_PREFIX: &str = "release-";

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct FetchProductTicketsArgs {
    /// The GitLab access token. This token requires at least `api:read` capabilities.
    #[clap(long, env = "GITLAB_TOKEN")]
    pub gitlab_token: String,

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

        let mut releases = read_release_file(&release_file)?;
        let series_nr = SeriesNumber::from(version);
        let release = releases
            .series
            .get_mut(&series_nr)
            .with_context(|| format!("Release series {series_nr} not found"))?
            .releases
            .get_mut(version)
            .with_context(|| format!("Release {version} not found in series {series_nr}"))?;

        release.tickets = self.fetch_product_tickets(version)?;

        tracing::info!(release = %version, tickets = release.tickets.len(), "Fetched product tickets");

        write_releases_file(release_file, releases)
    }

    fn fetch_product_tickets(&self, version: &Version) -> anyhow::Result<Vec<ProductTicket>> {
        let host = self
            .product_repo
            .host_str()
            .with_context(|| format!("No host part found in url {}", self.product_repo))?;
        let gitlab = Gitlab::new(host, &self.gitlab_token)?;

        let label = release_label(version);
        let project = self.product_repo.path().trim_matches('/');

        let endpoint = projects::issues::Issues::builder()
            .project(project)
            .label(label)
            .build()?;

        endpoint.query(&gitlab).context("Failed to fetch tickets")
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
