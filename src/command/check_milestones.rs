// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use clap::Args;
use jiff::Timestamp;
use owo_colors::OwoColorize as _;
use snafu::{ResultExt, Whatever};

use crate::{
    config::Config,
    gitlab_service::GitlabService,
    vcs_service::{IssueLinkType, Milestone, OverdueMilestone, VcsService},
};

#[derive(Clone, Debug, Args)]
pub(crate) struct CheckMilestonesArgs {
    #[arg(long)]
    faketime: Option<Timestamp>,
}

impl CheckMilestonesArgs {
    pub(crate) fn run(&self, config: &Config) -> Result<(), Whatever> {
        let gitlab_service = GitlabService::connect(
            config.gitlab_url.clone(),
            config.gitlab_token.clone(),
            config.gitlab_group.clone(),
        )?;

        let at = self.faketime.unwrap_or_else(Timestamp::now);

        Self::run_inner(&gitlab_service, at, &config.release_label)
    }

    fn run_inner(
        vcs_service: &dyn VcsService,
        at: Timestamp,
        release_label: &str,
    ) -> Result<(), Whatever> {
        let at = at.intz("UTC").whatever_context("invalid timestamp")?;

        let overdue_milestones =
            vcs_service.get_overdue_milestones_with_release_issues(at, release_label)?;

        println!("Found {} overdue milestones:", overdue_milestones.len());
        println!();
        for OverdueMilestone {
            milestone: Milestone { title, .. },
            overdue_since,
            issues,
        } in overdue_milestones
        {
            println!(
                "{}",
                format!("Milestone {}", title.green()).bold().underline()
            );
            println!();
            println!("overdue since {}", overdue_since.bold().blue(),);
            println!();
            println!(
                "{} open issues tagged with {}",
                issues.len().bold().blue(),
                release_label.bold().blue()
            );
            println!();
            for issue in issues {
                println!("- {}: {}", issue.short_reference.bold().blue(), issue.title);

                for linked_issue in issue.linked_issues {
                    if linked_issue.link_type == IssueLinkType::IsBlockedBy
                        && linked_issue.issue.state.is_opened()
                    {
                        println!(
                            "    → blocked by {}",
                            linked_issue.issue.short_reference.red()
                        );
                    }
                }
            }
            println!();
        }

        Ok(())
    }
}
