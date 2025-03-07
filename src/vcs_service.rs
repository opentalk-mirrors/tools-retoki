// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use jiff::{civil::Date, Timestamp, Zoned};
use semver::Version;
use snafu::{ResultExt as _, Whatever};

#[cfg_attr(test, mockall::automock)]
pub(crate) trait VcsService {
    fn get_milestones(&self) -> Result<Vec<Milestone>, Whatever>;

    fn get_open_issues_with_milestone_and_label(
        &self,
        milestone: &str,
        label: &str,
    ) -> Result<Vec<Issue>, Whatever>;

    fn get_overdue_milestones_with_release_issues(
        &self,
        at: Timestamp,
        release_label: &str,
    ) -> Result<Vec<OverdueMilestone>, Whatever> {
        let at = at.in_tz("UTC").whatever_context("invalid timestamp")?;

        let milestones = self.get_milestones()?;

        let overdue_milestones = milestones
            .into_iter()
            .filter(|m| m.title.parse::<Version>().is_ok())
            .filter_map(|m| {
                m.calculate_overdue_since(at.clone())
                    .map(|overdue_since| (m.id, m.title, overdue_since))
            })
            .collect::<Vec<_>>();

        let mut overdue_milestones_with_release_issues = Vec::new();

        for (id, title, overdue_since) in overdue_milestones {
            let issues = self.get_open_issues_with_milestone_and_label(&title, release_label)?;

            overdue_milestones_with_release_issues.push(OverdueMilestone {
                milestone: Milestone {
                    id,
                    title,
                    due_date: Some(overdue_since.date()),
                },
                overdue_since: overdue_since.timestamp(),
                issues,
            });
        }
        Ok(overdue_milestones_with_release_issues)
    }

    fn get_linked_issues(&self, project: &str, issue_id: u64)
        -> Result<Vec<LinkedIssue>, Whatever>;

    fn update_issue_description(
        &self,
        project: &str,
        issue_id: u64,
        description: &str,
    ) -> Result<(), Whatever>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Milestone {
    pub id: usize,
    pub title: String,
    pub due_date: Option<Date>,
}

impl Milestone {
    pub(crate) fn calculate_overdue_since(&self, at: Zoned) -> Option<Zoned> {
        match self
            .due_date
            .map(|d| d.at(0, 0, 0, 0).in_tz("UTC").expect("valid date"))
        {
            Some(due) if at >= due => Some(due),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Project {
    pub id: u64,
    pub path_with_namespace: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Issue {
    pub id: u64,
    pub iid: u64,
    pub title: String,
    pub project: Project,
    pub short_reference: String,
    pub description: Option<String>,
    pub state: IssueState,
    pub linked_issues: Vec<LinkedIssue>,
}

impl Issue {
    pub(crate) fn mermaid_identifier(&self) -> String {
        format!(
            "{}_{}",
            self.project
                .path_with_namespace
                .replace("/", "_")
                .replace("-", "_"),
            self.iid
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OverdueMilestone {
    pub milestone: Milestone,
    pub overdue_since: Timestamp,
    pub issues: Vec<Issue>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LinkedIssue {
    pub link_type: IssueLinkType,
    pub issue: Issue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum IssueLinkType {
    Blocks,
    IsBlockedBy,
    RelatesTo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum IssueState {
    Opened,
    Closed,
}

impl IssueState {
    pub(crate) const fn is_opened(&self) -> bool {
        matches!(self, Self::Opened)
    }
}
