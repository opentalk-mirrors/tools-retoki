use clap::Args;
use jiff::{Timestamp, Zoned};
use owo_colors::OwoColorize as _;
use snafu::Whatever;

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

        let at = self
            .faketime
            .map(|ts| ts.intz("UTC").expect("valid timestamp"))
            .unwrap_or_else(|| Zoned::now().intz("UTC").expect("valid datetime"));

        let overdue_milestones =
            gitlab_service.get_overdue_milestones_with_release_issues(at, &config.release_label)?;

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
                config.release_label.bold().blue()
            );
            println!();
            for issue in issues {
                let ticket_id = issue
                    .reference
                    .trim_start_matches(&format!("{}/", config.gitlab_group));
                println!("- {}: {}", ticket_id.bold().blue(), issue.title);

                for linked_issue in
                    gitlab_service.get_linked_issues(&issue.project.id.to_string(), issue.iid)?
                {
                    if linked_issue.link_type == IssueLinkType::IsBlockedBy
                        && linked_issue.issue.state.is_opened()
                    {
                        println!("    → blocked by {}", linked_issue.issue.reference);
                    }
                }
            }
            println!();
        }

        Ok(())
    }
}
