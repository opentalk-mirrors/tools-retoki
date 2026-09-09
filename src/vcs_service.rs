// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use owo_colors::{OwoColorize as _, Style};
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
    /// Fetch issues matching `filter` within `scope`.
    ///
    /// The returned issues carry an empty `linked_issues` list; call
    /// [`VcsService::get_linked_issues`] to load an issue's links when needed.
    fn fetch_issues<'a>(
        &self,
        scope: IssueScope<'a>,
        filter: &IssueFilter<'a>,
    ) -> anyhow::Result<Vec<Issue>>;

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
    fn fetch_issues(
        &self,
        scope: IssueScope<'_>,
        filter: &IssueFilter<'_>,
    ) -> anyhow::Result<Vec<Issue>> {
        self.inner.fetch_issues(scope, filter)
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

    /// Match a release issue among `candidates`, preferring an exact
    /// `expected_title` match and falling back to any title containing
    /// `version_marker`.
    pub(crate) fn match_release_issue<'a, I>(
        candidates: I,
        expected_title: &str,
        version_marker: &str,
    ) -> Option<&'a Issue>
    where
        I: IntoIterator<Item = &'a Issue>,
    {
        let candidates: Vec<&Issue> = candidates.into_iter().collect();

        if let Some(exact) = candidates
            .iter()
            .copied()
            .find(|i| i.title == expected_title)
        {
            return Some(exact);
        }

        let fuzzy = candidates
            .into_iter()
            .find(|i| title_matches_version(&i.title, version_marker))?;
        tracing::warn!(
            existing_title = %fuzzy.title,
            expected_title,
            "matched existing component release issue by version substring in title; \
             consider renaming the ticket to the expected title",
        );
        Some(fuzzy)
    }
}

/// Whether `title` contains `version_marker` as a standalone version rather
/// than a prefix of a longer one, so that `26.1.1` doesn't match inside
/// `26.1.1-beta.1`.
fn title_matches_version(title: &str, version_marker: &str) -> bool {
    title.match_indices(version_marker).any(|(idx, matched)| {
        let next = title[idx + matched.len()..].chars().next();
        !next.is_some_and(is_version_continuation)
    })
}

/// Characters that would extend a semver version to the right (further digits,
/// a patch separator, a pre-release, or build metadata).
fn is_version_continuation(c: char) -> bool {
    c.is_ascii_digit() || matches!(c, '.' | '-' | '+')
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

/// Selects the set of projects that [`VcsService::fetch_issues`] searches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IssueScope<'a> {
    /// All projects within the service's configured group.
    Group,
    /// A single project identified by its path.
    Project(&'a str),
}

/// Optional filters applied by [`VcsService::fetch_issues`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct IssueFilter<'a> {
    /// Restrict to issues carrying all of these labels.
    pub labels: &'a [&'a str],
    /// Restrict to issues in this state.
    pub state: Option<IssueState>,
}

fn print_description_diff(current: Option<&str>, next: &str) {
    let old = current.unwrap_or_default();

    if old == next {
        return;
    }

    let diff = TextDiff::from_lines(old, next);

    println!("{}", "--- current".red());
    println!("{}", "+++ new".green());
    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            println!("{:-^1$}", "-", 80);
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, style) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::new().red()),
                    ChangeTag::Insert => ("+", Style::new().green()),
                    ChangeTag::Equal => (" ", Style::new()),
                };
                print!("{}", sign.style(style));
                for (emphasized, value) in change.iter_strings_lossy() {
                    let style = if emphasized { style.underline() } else { style };
                    print!("{}", value.style(style));
                }
                if change.missing_newline() {
                    println!();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn issue(title: &str) -> Issue {
        Issue {
            id: 1,
            iid: 1,
            title: title.to_owned(),
            project: Project {
                id: 1,
                path_with_namespace: "group/project".to_owned(),
            },
            short_reference: "group/project#1".to_owned(),
            description: None,
            state: IssueState::Opened,
            linked_issues: Vec::new(),
            web_url: Url::parse("https://example.com/group/project/-/issues/1").unwrap(),
        }
    }

    #[test]
    fn match_release_issue_prefers_exact_title() {
        let fuzzy = issue("Release v26.1.1 of Controller and more");
        let exact = issue("Release v26.1.1 of Controller");
        let candidates = [fuzzy.clone(), exact.clone(), fuzzy.clone()];

        let found =
            Issue::match_release_issue(&candidates, "Release v26.1.1 of Controller", "26.1.1");

        assert_eq!(found, Some(&exact));
    }

    #[test]
    fn match_release_issue_falls_back_to_version_substring() {
        let candidate = issue("Bump to v26.1.1 please");
        let candidates = [candidate.clone()];

        let found =
            Issue::match_release_issue(&candidates, "Release v26.1.1 of Controller", "26.1.1");

        assert_eq!(found, Some(&candidate));
    }

    #[test]
    fn match_release_issue_returns_none_when_nothing_matches() {
        let candidates = [issue("Release v25.0.0 of Controller")];

        let found =
            Issue::match_release_issue(&candidates, "Release v26.1.1 of Controller", "26.1.1");

        assert_eq!(found, None);
    }

    #[test]
    fn match_release_issue_does_not_match_prerelease_for_stable_version() {
        // Regression: the stable marker "26.1.1" must not fuzzy-match a beta
        // release issue whose version is "26.1.1-beta.1".
        let beta = issue("Release v26.1.1-beta.1 of Controller");
        let candidates = [beta];

        let found =
            Issue::match_release_issue(&candidates, "Release v26.1.1 of Controller", "26.1.1");

        assert_eq!(
            found, None,
            "stable version must not match a prerelease issue by substring",
        );
    }

    #[test]
    fn match_release_issue_matches_the_matching_prerelease_marker() {
        // A beta marker still matches its own beta issue, even when a stable
        // issue for the same base version is also present.
        let stable = issue("Release v26.1.1 of Controller");
        let beta = issue("Release v26.1.1-beta.1 of Controller");
        let candidates = [stable, beta.clone()];

        let found = Issue::match_release_issue(
            &candidates,
            "Release v26.1.1-beta.1 of Controller",
            "26.1.1-beta.1",
        );

        assert_eq!(found, Some(&beta));
    }

    #[test]
    fn title_matches_version_rejects_longer_versions() {
        assert!(title_matches_version(
            "Release v26.1.1 of Controller",
            "26.1.1"
        ));
        assert!(title_matches_version("v26.1.1", "26.1.1"));
        assert!(!title_matches_version("Release v26.1.1-beta.1", "26.1.1"));
        assert!(!title_matches_version("Release v26.1.10", "26.1.1"));
        assert!(!title_matches_version("Release v26.1.1.4", "26.1.1"));
        assert!(!title_matches_version("Release v26.1.1+build", "26.1.1"));
    }
}
