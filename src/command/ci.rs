// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::io::stdout;

use clap::Args;
use owo_colors::OwoColorize as _;
use url::Url;

use crate::{
    bot_config::Config,
    gitlab_service::GitlabService,
    output::Output,
    tasks::generate_dependency_graph::DependencyGraphUpdater,
    vcs_service::{VcsService, VcsServiceExt as _},
};

#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct CiArgs {
    /// Only log the actions that would be performed without writing anything.
    #[arg(long, env = "RETOKI_DRY_RUN")]
    dry_run: bool,
}

impl CiArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        let config = Config::load()?;
        let gitlab_service = GitlabService::connect(
            config.gitlab_url.clone(),
            config.gitlab_token.clone(),
            config.gitlab_group.clone(),
        )?
        .dry_run_if(self.dry_run);

        Self::run_inner(
            &gitlab_service,
            &config.release_label,
            &mut stdout().lock(),
            self.dry_run,
            &config.gitlab_url,
        )
    }

    fn run_inner(
        vcs_service: &dyn VcsService,
        release_label: &str,
        out: &mut dyn Output,
        dry_run: bool,
        base_url: &Url,
    ) -> anyhow::Result<()> {
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
