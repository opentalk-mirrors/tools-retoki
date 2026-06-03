// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Context as _;
use jiff::{civil::Date, Timestamp, Zoned};
use rayon::iter::{IntoParallelIterator as _, ParallelIterator as _};
use semver::Version;

/// Extension trait that lets any `VcsService` be wrapped in the available
/// decorators via chainable, statically-dispatched combinators.
pub(crate) trait VcsServiceExt: VcsService + Sized {
    /// Wrap in [`DryRunVcsService`] whose dry-run interception is toggled by
    /// `enabled`; when `false`, writes pass straight through to `self`.
    fn dry_run_if(self, enabled: bool) -> DryRunVcsService<Self> {
        DryRunVcsService::with_enabled(self, enabled)
    }
}

impl<T: VcsService> VcsServiceExt for T {}

#[cfg_attr(test, mockall::automock)]
pub(crate) trait VcsService: Send + Sync {
    fn get_milestones(&self) -> anyhow::Result<Vec<Milestone>>;

    fn get_open_issues_with_label(&self, label: &str) -> anyhow::Result<Vec<Issue>>;

    fn get_open_issues_with_milestone_and_label(
        &self,
        milestone: &str,
        label: &str,
    ) -> anyhow::Result<Vec<Issue>>;

    fn get_overdue_milestones_with_release_issues(
        &self,
        at: Timestamp,
        release_label: &str,
    ) -> anyhow::Result<Vec<OverdueMilestone>> {
        let at = at.in_tz("UTC").context("invalid timestamp")?;

        let milestones = self.get_milestones()?;

        let overdue_milestones = milestones
            .into_iter()
            .filter(|m| m.title.parse::<Version>().is_ok())
            .filter_map(|m| {
                m.calculate_overdue_since(at.clone())
                    .map(|overdue_since| (m.id, m.title, overdue_since))
            })
            .collect::<Vec<_>>();

        overdue_milestones
            .into_par_iter()
            .map(|(id, title, overdue_since)| {
                let issues =
                    self.get_open_issues_with_milestone_and_label(&title, release_label)?;
                Ok(OverdueMilestone {
                    milestone: Milestone {
                        id,
                        title,
                        due_date: Some(overdue_since.date()),
                    },
                    overdue_since: overdue_since.timestamp(),
                    issues,
                })
            })
            .collect()
    }

    fn get_linked_issues(&self, project: &str, issue_id: u64) -> anyhow::Result<Vec<LinkedIssue>>;

    fn update_issue_description(
        &self,
        project: &str,
        issue_id: u64,
        description: &str,
    ) -> anyhow::Result<()>;
}

/// `VcsService` decorator for dry-run mode: read operations are delegated to
/// the wrapped service, write operations are logged via `tracing::info!` and
/// return synthetic values.
#[derive(Debug)]
pub(crate) struct DryRunVcsService<S> {
    inner: S,
    enabled: bool,
}

impl<S> DryRunVcsService<S> {
    pub(crate) fn with_enabled(inner: S, enabled: bool) -> Self {
        Self { inner, enabled }
    }
}

impl<S: VcsService> VcsService for DryRunVcsService<S> {
    fn get_milestones(&self) -> anyhow::Result<Vec<Milestone>> {
        self.inner.get_milestones()
    }

    fn get_open_issues_with_label(&self, label: &str) -> anyhow::Result<Vec<Issue>> {
        self.inner.get_open_issues_with_label(label)
    }

    fn get_open_issues_with_milestone_and_label(
        &self,
        milestone: &str,
        label: &str,
    ) -> anyhow::Result<Vec<Issue>> {
        self.inner
            .get_open_issues_with_milestone_and_label(milestone, label)
    }

    fn get_linked_issues(&self, project: &str, issue_id: u64) -> anyhow::Result<Vec<LinkedIssue>> {
        self.inner.get_linked_issues(project, issue_id)
    }

    fn update_issue_description(
        &self,
        project: &str,
        issue_id: u64,
        description: &str,
    ) -> anyhow::Result<()> {
        if self.enabled {
            tracing::info!(project, issue_id, "DRY RUN: would update issue description",);
            Ok(())
        } else {
            self.inner
                .update_issue_description(project, issue_id, description)
        }
    }
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
