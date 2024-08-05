// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{
    fmt::Arguments,
    io::{stdout, Write},
};

use clap::Args;
use jiff::Timestamp;
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

trait Output {
    fn println(&mut self, value: &Arguments);
}

impl<W: Write> Output for W {
    fn println(&mut self, value: &Arguments) {
        let _ = writeln!(self, "{}", value);
    }
}

impl CheckMilestonesArgs {
    pub(crate) fn run(&self, config: &Config) -> Result<(), Whatever> {
        let gitlab_service = GitlabService::connect(
            config.gitlab_url.clone(),
            config.gitlab_token.clone(),
            config.gitlab_group.clone(),
        )?;

        let at = self.faketime.unwrap_or_else(Timestamp::now);

        Self::run_inner(
            &gitlab_service,
            at,
            &config.release_label,
            &mut stdout().lock(),
        )
    }

    fn run_inner(
        vcs_service: &dyn VcsService,
        at: Timestamp,
        release_label: &str,
        out: &mut dyn Output,
    ) -> Result<(), Whatever> {
        let overdue_milestones =
            vcs_service.get_overdue_milestones_with_release_issues(at, release_label)?;

        out.println(&format_args!(
            "Found {} overdue milestones",
            overdue_milestones.len()
        ));
        for OverdueMilestone {
            milestone: Milestone { title, .. },
            overdue_since,
            issues,
        } in overdue_milestones
        {
            out.println(&format_args!(""));
            out.println(&format_args!(
                "{}",
                format!("Milestone {}", title.green()).bold().underline()
            ));
            out.println(&format_args!(""));
            out.println(&format_args!(
                "overdue since {}",
                overdue_since.bold().blue()
            ));
            out.println(&format_args!(""));
            out.println(&format_args!(
                "{} open issues tagged with {}",
                issues.len().bold().blue(),
                release_label.bold().blue()
            ));
            if !issues.is_empty() {
                out.println(&format_args!(""));
            }
            for issue in issues {
                out.println(&format_args!(
                    "- {}: {}",
                    issue.short_reference.bold().blue(),
                    issue.title
                ));

                for linked_issue in issue.linked_issues {
                    if linked_issue.link_type == IssueLinkType::IsBlockedBy
                        && linked_issue.issue.state.is_opened()
                    {
                        out.println(&format_args!(
                            "    → blocked by {}",
                            linked_issue.issue.short_reference.red()
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use jiff::{civil::Date, Timestamp};
    use mockall::predicate::eq;
    use pretty_assertions::assert_eq;
    use strip_ansi_escapes::strip_str;

    use super::{CheckMilestonesArgs, Output};
    use crate::vcs_service::{
        Issue, IssueState, LinkedIssue, Milestone, MockVcsService, OverdueMilestone, Project,
    };

    struct DummyOutput(String);

    impl DummyOutput {
        fn new() -> Self {
            Self(String::new())
        }
    }

    impl Output for DummyOutput {
        fn println(&mut self, value: &std::fmt::Arguments) {
            self.0.push_str(&format!("{}\n", value));
        }
    }

    #[test]
    fn check_milestones_empty() {
        let mut vcs_service_mock = MockVcsService::new();
        let mut output = DummyOutput::new();

        let at = Timestamp::from_second(1722866956).unwrap();

        let _ = vcs_service_mock
            .expect_get_overdue_milestones_with_release_issues()
            .with(eq(at), eq("Release"))
            .times(1)
            .return_once(|_, _| Ok(vec![]));

        CheckMilestonesArgs::run_inner(&vcs_service_mock, at, "Release", &mut output).unwrap();

        assert_eq!(
            "\
Found 0 overdue milestones
",
            output.0
        );
    }

    #[test]
    fn check_one_overdue_milestone_without_issues() {
        let mut vcs_service_mock = MockVcsService::new();
        let mut output = DummyOutput::new();

        let at = Timestamp::from_second(1722866956).unwrap();

        let _ = vcs_service_mock
            .expect_get_overdue_milestones_with_release_issues()
            .with(eq(at), eq("Release"))
            .times(1)
            .return_once(|_, _| {
                Ok(vec![OverdueMilestone {
                    milestone: Milestone {
                        id: 780,
                        title: "24.10.0".to_string(),
                        due_date: Some(Date::constant(2024, 5, 10)),
                    },
                    overdue_since: Timestamp::from_second(1715292000).unwrap(),
                    issues: vec![],
                }])
            });

        CheckMilestonesArgs::run_inner(&vcs_service_mock, at, "Release", &mut output).unwrap();

        assert_eq!(
            "\
Found 1 overdue milestones

Milestone 24.10.0

overdue since 2024-05-09T22:00:00Z

0 open issues tagged with Release
",
            strip_str(output.0)
        );
    }

    #[test]
    fn check_multiple_overdue_milestones_with_issues() {
        let mut vcs_service_mock = MockVcsService::new();
        let mut output = DummyOutput::new();

        let at = Timestamp::from_second(1722866956).unwrap();

        let _ = vcs_service_mock
            .expect_get_overdue_milestones_with_release_issues()
            .with(eq(at), eq("Release"))
            .times(1)
            .return_once(|_, _| {
                Ok(vec![
                    OverdueMilestone {
                        milestone: Milestone {
                            id: 780,
                            title: "24.10.0".to_string(),
                            due_date: Some(Date::constant(2024, 5, 10)),
                        },
                        overdue_since: Timestamp::from_second(1715292000).unwrap(),
                        issues: vec![Issue {
                            id: 423,
                            iid: 49,
                            title: "Release v1.2.3 of component".to_string(),
                            project: Project {
                                id: 283,
                                path_with_namespace: "path/to/project".to_string(),
                            },
                            short_reference: "to/project#49".to_string(),
                            state: IssueState::Opened,
                            linked_issues: vec![],
                        }],
                    },
                    OverdueMilestone {
                        milestone: Milestone {
                            id: 785,
                            title: "24.11.0-rc.1".to_string(),
                            due_date: Some(Date::constant(2024, 6, 1)),
                        },
                        overdue_since: Timestamp::from_second(1715292000).unwrap(),
                        issues: vec![Issue {
                            id: 425,
                            iid: 52,
                            title: "Release v1.3.0 of component".to_string(),
                            project: Project {
                                id: 283,
                                path_with_namespace: "path/to/project".to_string(),
                            },
                            short_reference: "to/project#52".to_string(),
                            state: IssueState::Opened,
                            linked_issues: vec![LinkedIssue {
                                link_type: crate::vcs_service::IssueLinkType::IsBlockedBy,
                                issue: Issue {
                                    id: 95,
                                    iid: 93,
                                    title: "Fix issue abc".to_string(),
                                    project: Project {
                                        id: 22,
                                        path_with_namespace: "path/to/another/project".to_string(),
                                    },
                                    short_reference: "to/another/project#93".to_string(),
                                    state: IssueState::Opened,
                                    linked_issues: vec![],
                                },
                            }],
                        }],
                    },
                ])
            });

        CheckMilestonesArgs::run_inner(&vcs_service_mock, at, "Release", &mut output).unwrap();

        assert_eq!(
            "\
Found 2 overdue milestones

Milestone 24.10.0

overdue since 2024-05-09T22:00:00Z

1 open issues tagged with Release

- to/project#49: Release v1.2.3 of component

Milestone 24.11.0-rc.1

overdue since 2024-05-09T22:00:00Z

1 open issues tagged with Release

- to/project#52: Release v1.3.0 of component
    → blocked by to/another/project#93
",
            strip_str(output.0)
        );
    }
}
