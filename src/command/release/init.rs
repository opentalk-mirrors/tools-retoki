// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
use std::io::stdout;

use clap::Args;
use semver::Version;

use crate::{
    bot_config::Config,
    bot_templates,
    command::ProfileArgs,
    gitlab_service::GitlabService,
    output::Output,
    release_workflow::ReleasesBuilder,
    tasks::generate_dependency_graph::dependency_graph_for_issue,
    vcs_service::{VcsService, VcsServiceExt},
};

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct InitArgs {
    /// Only log the actions that would be performed without writing anything.
    #[arg(long, env = "RETOKI_DRY_RUN")]
    dry_run: bool,

    #[clap(flatten)]
    pub profile_args: ProfileArgs,
}

impl InitArgs {
    pub fn execute(&self, version: &Version) -> anyhow::Result<()> {
        let config = Config::load()?;
        let vcs = GitlabService::connect(
            config.gitlab_url.clone(),
            config.gitlab_token.clone(),
            config.gitlab_group.clone(),
        )?
        .dry_run_if(self.dry_run);

        self.run_inner(version, &config, &vcs, &mut stdout().lock())
    }

    fn run_inner(
        &self,
        version: &Version,
        config: &Config,
        vcs: &dyn VcsService,
        out: &mut dyn Output,
    ) -> anyhow::Result<()> {
        let mut releases = ReleasesBuilder::new()
            .dry_run(self.dry_run)
            .load(config.releases_yml_path.clone(), &self.profile_args.profile)?;
        let release = releases
            .edit(|r| r.get_or_insert_from_previous(version.clone()))?
            .save()?;

        let title = format!("Release {version}");
        let issue = vcs.get_open_issue_with_title(&config.release_repo, &title)?;
        let linked_issues = issue
            .as_ref()
            .map(|issue| issue.linked_issues.as_slice())
            .unwrap_or(&[]);
        let categories: Vec<_> = releases
            .category_data(version, &release, linked_issues, vcs)?
            .collect();

        if let Some(issue) = issue {
            let dependency_graph = dependency_graph_for_issue(vcs, &issue, &config.gitlab_url)?;
            let body = bot_templates::product_release_body(
                vcs,
                &config.release_repo,
                version,
                &categories,
                &dependency_graph,
            )?;
            vcs.update_issue_description(&config.release_repo, issue.iid, &body)?;
            out.println(&format_args!(
                "Updated description of issue {}",
                issue.reference_with_url()
            ));
        } else {
            // The dependency graph is rooted at the release issue, so it can
            // only be rendered once the issue exists and its id is known.
            let body = bot_templates::product_release_body(
                vcs,
                &config.release_repo,
                version,
                &categories,
                "",
            )?;
            let created = vcs.create_issue(
                &config.release_repo,
                &title,
                &body,
                &[&config.release_label],
            )?;
            out.println(&format_args!(
                "Created issue {}",
                created.reference_with_url()
            ));
            let dependency_graph = dependency_graph_for_issue(vcs, &created, &config.gitlab_url)?;
            let body = bot_templates::product_release_body(
                vcs,
                &config.release_repo,
                version,
                &categories,
                &dependency_graph,
            )?;
            vcs.update_issue_description(&config.release_repo, created.iid, &body)?;
        }

        Ok(())
    }
}
