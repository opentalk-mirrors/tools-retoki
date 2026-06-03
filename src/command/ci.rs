// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::io::stdout;

use clap::Args;
use jiff::Timestamp;
use owo_colors::OwoColorize as _;
use url::Url;

use crate::{
    cli::CommonArgs,
    config::Config,
    gitlab_service::GitlabService,
    output::Output,
    tasks::{
        generate_dependency_graph::DependencyGraphUpdater,
        overdue_milestones::list_overdue_milestones,
    },
    vcs_service::{VcsService, VcsServiceExt as _},
};

#[derive(Clone, Debug, Args)]
pub(crate) struct CiArgs {
    #[arg(long)]
    faketime: Option<Timestamp>,
}

impl CiArgs {
    pub(crate) fn run(&self, common_args: &CommonArgs, config: &Config) -> anyhow::Result<()> {
        let gitlab_service = GitlabService::connect(
            config.gitlab_url.clone(),
            config.gitlab_token.clone(),
            config.gitlab_group.clone(),
        )?
        .dry_run_if(common_args.dry_run);

        let at = self.faketime.unwrap_or_else(Timestamp::now);

        Self::run_inner(
            &gitlab_service,
            at,
            &config.release_label,
            &mut stdout().lock(),
            common_args.dry_run,
            &config.gitlab_url,
        )
    }

    fn run_inner(
        vcs_service: &dyn VcsService,
        at: Timestamp,
        release_label: &str,
        out: &mut dyn Output,
        dry_run: bool,
        base_url: &Url,
    ) -> anyhow::Result<()> {
        out.println(&format_args!(
            "{}",
            "TASK: Listing overdue milestones…"
                .black()
                .on_bright_blue()
                .bold()
        ));
        list_overdue_milestones(vcs_service, at, release_label, out)?;
        out.println(&format_args!(""));

        out.println(&format_args!(
            "{}",
            "TASK: Updating dependency graphs…"
                .black()
                .on_bright_blue()
                .bold()
        ));
        DependencyGraphUpdater::new(vcs_service, release_label, out, base_url, dry_run).apply()?;
        Ok(())
    }
}
