// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
use std::io::stdout;

use clap::Args;

use crate::{
    cli::CommonArgs,
    command::release::ReleaseArgs,
    config::Config,
    gitlab_service::GitlabService,
    output::Output,
    releases::ReleasesBuilder,
    templates,
    vcs_service::{Issue, VcsService, VcsServiceExt},
};

#[derive(Debug, Clone, Args)]
pub struct InitArgs;

impl InitArgs {
    pub fn run(
        &self,
        release_args: &ReleaseArgs,
        common_args: &CommonArgs,
        config: &Config,
    ) -> anyhow::Result<()> {
        let vcs = GitlabService::connect(
            config.gitlab_url.clone(),
            config.gitlab_token.clone(),
            config.gitlab_group.clone(),
        )?
        .dry_run_if(common_args.dry_run);

        self.run_inner(
            release_args,
            common_args,
            config,
            &vcs,
            &mut stdout().lock(),
        )
    }

    fn run_inner(
        &self,
        release_args: &ReleaseArgs,
        common_args: &CommonArgs,
        config: &Config,
        vcs: &dyn VcsService,
        out: &mut dyn Output,
    ) -> anyhow::Result<()> {
        let mut releases = ReleasesBuilder::new()
            .dry_run(common_args.dry_run)
            .load(config.releases_yml_path.clone(), &config.release_profile)?;
        let release = releases
            .edit(|r| r.get_or_insert_from_previous(release_args.version.clone()))?
            .save()?;

        let title = release_args.title();
        let issue = vcs.get_open_issue_with_title(&config.release_repo, &title)?;
        let linked_issues = issue
            .as_ref()
            .map(|issue| issue.linked_issues.as_slice())
            .unwrap_or(&[]);
        let categories: Vec<_> = releases
            .category_data(&release_args.version, &release, linked_issues, vcs)
            .collect();
        let body = templates::product_release_body(
            vcs,
            &config.release_repo,
            &release_args.version,
            &categories,
        )?;

        if let Some(Issue {
            iid,
            project,
            web_url,
            ..
        }) = issue
        {
            vcs.update_issue_description(&config.release_repo, iid, &body)?;
            out.println(&format_args!(
                "Updated description of issue {path}{iid}\n 🌐 {web_url}",
                path = project.path_with_namespace
            ));
        } else {
            let created = vcs.create_issue(
                &config.release_repo,
                &title,
                &body,
                &[&config.release_label],
            )?;
            out.println(&format_args!(
                "Created issue {path}{iid}\n 🌐 {web_url}",
                path = created.project.path_with_namespace,
                iid = created.iid,
                web_url = created.web_url
            ));
        }

        Ok(())
    }
}
