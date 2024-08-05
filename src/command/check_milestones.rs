use clap::Args;
use jiff::{Timestamp, Zoned};
use owo_colors::OwoColorize as _;
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
                let full_path = issue
                    .project
                    .path_with_namespace
                    .trim_start_matches(&format!("{}/", config.gitlab_group));
                let ticket_id = format!("{}#{}", full_path, issue.id.to_string().bold());
                println!("- {}: {}", ticket_id.blue(), issue.title);
            }
            println!();
        }

        Ok(())
    }
}
