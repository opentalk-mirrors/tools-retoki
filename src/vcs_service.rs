// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

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
    fn get_open_issues_with_label(&self, label: &str) -> anyhow::Result<Vec<Issue>>;

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
    fn get_open_issues_with_label(&self, label: &str) -> anyhow::Result<Vec<Issue>> {
        self.inner.get_open_issues_with_label(label)
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
