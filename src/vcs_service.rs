// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use owo_colors::OwoColorize as _;
use similar::{ChangeTag, TextDiff};
use url::Url;

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

    /// Read an issue.
    fn get_issue(&self, project: &str, issue_id: u64) -> anyhow::Result<Option<Issue>>;

    fn update_issue_description(
        &self,
        project: &str,
        issue_id: u64,
        description: &str,
    ) -> anyhow::Result<()>;

    /// Find an open issue in `project` whose title exactly matches `title`,
    /// returning `None` if no such issue exists.
    fn get_open_issue_with_title(
        &self,
        project: &str,
        title: &str,
    ) -> anyhow::Result<Option<Issue>>;

    /// Create a new issue in `project` and return the created issue.
    #[expect(clippy::needless_lifetimes)]
    fn create_issue<'a>(
        &self,
        project: &str,
        title: &str,
        description: &str,
        labels: &[&'a str],
    ) -> anyhow::Result<Issue>;

    /// List issues in `project` carrying the given `label`, regardless of their
    /// state (open or closed).
    fn find_issues_with_label(&self, project: &str, label: &str) -> anyhow::Result<Vec<Issue>>;

    /// Create a link of the given `link_type` from the issue `source_iid` in
    /// `source_project` to the issue `target_iid` in `target_project`.
    fn create_issue_link(
        &self,
        source_project: &str,
        source_iid: u64,
        target_project: &str,
        target_iid: u64,
        link_type: IssueLinkType,
    ) -> anyhow::Result<()>;

    /// Read a raw file from the repository.
    ///
    /// Returns `Ok(None)` if the file does not exist.
    fn get_raw_file(&self, project: &str, path: &str) -> anyhow::Result<Option<String>>;

    fn project_path_from_url(&self, url: &Url) -> anyhow::Result<String>;
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

    fn get_issue(&self, project: &str, issue_id: u64) -> anyhow::Result<Option<Issue>> {
        self.inner.get_issue(project, issue_id)
    }

    fn update_issue_description(
        &self,
        project: &str,
        issue_id: u64,
        description: &str,
    ) -> anyhow::Result<()> {
        if self.enabled {
            let current_description = self
                .inner
                .get_issue(project, issue_id)?
                .ok_or_else(|| anyhow::anyhow!("issue {issue_id} in project {project} not found"))?
                .description;
            println!("Would update issue {}#{}:", project, issue_id);
            print_description_diff(current_description.as_deref(), description);

            tracing::info!(project, issue_id, "DRY RUN: would update issue description",);
            Ok(())
        } else {
            self.inner
                .update_issue_description(project, issue_id, description)
        }
    }

    fn get_open_issue_with_title(
        &self,
        project: &str,
        title: &str,
    ) -> anyhow::Result<Option<Issue>> {
        self.inner.get_open_issue_with_title(project, title)
    }

    fn create_issue(
        &self,
        project: &str,
        title: &str,
        description: &str,
        labels: &[&str],
    ) -> anyhow::Result<Issue> {
        if self.enabled {
            tracing::info!(project, title, ?labels, "DRY RUN: would create issue");
            Ok(Issue {
                id: 0,
                iid: 0,
                title: title.to_owned(),
                project: Project {
                    id: 0,
                    path_with_namespace: project.to_owned(),
                },
                short_reference: format!("{project}#0"),
                description: Some(description.to_owned()),
                state: IssueState::Opened,
                linked_issues: Vec::new(),
                web_url: Url::parse(&format!("https://git.opentalk.dev/{project}/-/issues/0"))
                    .expect("Hardcoded URL should be valid"),
            })
        } else {
            self.inner.create_issue(project, title, description, labels)
        }
    }

    fn find_issues_with_label(&self, project: &str, label: &str) -> anyhow::Result<Vec<Issue>> {
        self.inner.find_issues_with_label(project, label)
    }

    fn create_issue_link(
        &self,
        source_project: &str,
        source_iid: u64,
        target_project: &str,
        target_iid: u64,
        link_type: IssueLinkType,
    ) -> anyhow::Result<()> {
        if self.enabled {
            tracing::info!(
                source_project,
                source_iid,
                target_project,
                target_iid,
                ?link_type,
                "DRY RUN: would create issue link",
            );
            Ok(())
        } else {
            self.inner.create_issue_link(
                source_project,
                source_iid,
                target_project,
                target_iid,
                link_type,
            )
        }
    }

    fn get_raw_file(&self, project: &str, path: &str) -> anyhow::Result<Option<String>> {
        self.inner.get_raw_file(project, path)
    }

    fn project_path_from_url(&self, url: &Url) -> anyhow::Result<String> {
        self.inner.project_path_from_url(url)
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
    pub web_url: Url,
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

    /// Render the issue as `path#iid` followed by its web URL, for printing in command output.
    pub(crate) fn reference_with_url(&self) -> String {
        format!(
            "{path}#{iid}\n 🌐 {url}",
            path = self.project.path_with_namespace,
            iid = self.iid,
            url = self.web_url,
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

fn print_description_diff(current: Option<&str>, next: &str) {
    let old = current.unwrap_or_default();

    if old == next {
        return;
    }

    let diff = TextDiff::from_lines(old, next);

    println!("{}", "--- current".red());
    println!("{}", "+++ new".green());
    for hunk in diff.unified_diff().iter_hunks() {
        println!("{}", hunk.header().cyan());
        for change in hunk.iter_changes() {
            let line = change.value().strip_suffix('\n').unwrap_or(change.value());
            match change.tag() {
                ChangeTag::Delete => println!("{}", format_args!("-{line}").red()),
                ChangeTag::Insert => println!("{}", format_args!("+{line}").green()),
                ChangeTag::Equal => println!(" {line}"),
            }
        }
    }
}
