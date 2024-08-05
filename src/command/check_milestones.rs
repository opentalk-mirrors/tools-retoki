use clap::Args;
use jiff::{Timestamp, Zoned};
use snafu::Whatever;

use crate::{
    config::Config,
    gitlab_service::GitlabService,
    vcs_service::{Milestone, OverdueMilestone, VcsService},
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
                "{title} is overdue since {overdue_since} with {} open release issues",
                issues.len()
            );
            for issue in issues {
                println!(
                    "- {}#{}: {}",
                    issue.project.path_with_namespace, issue.id, issue.title
                );
            }
            println!();
        }

        Ok(())
    }
}
