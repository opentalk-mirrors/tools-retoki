// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

mod api;

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
};

use anyhow::{Context as _, bail};
use derive_builder::Builder;
use gitlab::{
    Gitlab,
    api::{Endpoint, ParamValue, Query as _, QueryParams, common::NameOrId, issues::IssueState},
};
use http::Method;
use rayon::iter::{IntoParallelIterator, ParallelIterator as _};
use serde::Serialize;
use url::Url;

use self::api::LinkedItem;
use crate::{
    gitlab_service::api::{Issue, Project},
    vcs_service::{self, IssueFilter, IssueScope, VcsService},
};

pub(crate) struct GitlabService {
    group: String,
    client: Gitlab,
}

impl GitlabService {
    pub(crate) fn connect(url: Url, token: String, group: String) -> anyhow::Result<Self> {
        let host = url
            .as_str()
            .trim_start_matches(&format!("{}://", url.scheme()));
        let client = Gitlab::new(host, token)
            .with_context(|| format!("failed to create gitlab client for {}", host))?;

        Ok(Self { group, client })
    }

    fn get_projects(&self, project_ids: BTreeSet<u64>) -> anyhow::Result<BTreeMap<u64, Project>> {
        project_ids
            .into_par_iter()
            .map(|id| Ok((id, self.get_project(id)?)))
            .collect()
    }

    fn get_project(&self, id: u64) -> anyhow::Result<Project> {
        gitlab::api::projects::Project::builder()
            .project(id)
            .build()
            .with_context(|| format!("failed to build endpoint for project {id}"))?
            .query(&self.client)
            .with_context(|| format!("failed to fetch project {id}"))
    }
}

impl VcsService for GitlabService {
    #[tracing::instrument(level = "info", skip(self), err)]
    fn fetch_issues(
        &self,
        scope: IssueScope<'_>,
        filter: &IssueFilter<'_>,
    ) -> anyhow::Result<Vec<vcs_service::Issue>> {
        let state = filter.state.map(issue_state);

        let issues: Vec<Issue> = match scope {
            IssueScope::Group => {
                let endpoint = Issues::builder(&self.group)
                    .state(state)
                    .labels(filter.labels)
                    .build()
                    .context("couldn't build group issues endpoint")?;
                endpoint
                    .query(&self.client)
                    .context("couldn't query group issues")?
            }
            IssueScope::Project(project) => {
                let mut builder = gitlab::api::projects::issues::Issues::builder();
                let _ = builder.project(project);
                if !filter.labels.is_empty() {
                    let _ = builder.labels(filter.labels.iter().copied());
                }
                if let Some(state) = state {
                    let _ = builder.state(state);
                }
                let endpoint = builder
                    .build()
                    .context("couldn't build project issues endpoint")?;
                endpoint
                    .query(&self.client)
                    .with_context(|| format!("couldn't query issues in project {project}"))?
            }
        };

        let project_ids = issues.iter().map(|i| i.project_id).collect();
        let projects = self.get_projects(project_ids)?;

        // Links are intentionally not resolved here; callers that need an
        // issue's blockers fetch them explicitly via `get_linked_issues`.
        issues
            .into_iter()
            .map(|i| {
                i.to_vcs_service_issue(
                    &projects,
                    &self.group,
                    vcs_service::LinkedIssues::NotFetched,
                )
            })
            .collect()
    }

    #[tracing::instrument(level = "info", skip(self), err)]
    fn get_linked_issues(
        &self,
        project: &str,
        issue_id: u64,
    ) -> anyhow::Result<Vec<vcs_service::LinkedIssue>> {
        let endpoint = LinkedItems::builder()
            .project(NameOrId::Name(project.into()))
            .issue(issue_id)
            .build()
            .context("couldn't build linked issues endpoint")?;

        let linked_items: Vec<LinkedItem> = endpoint.query(&self.client).with_context(|| {
            format!("couldn't get linked issues for issue {issue_id} in project {project}")
        })?;

        let project_ids = linked_items
            .iter()
            .filter_map(|i| match &i.item {
                api::IssueOrEpic::Issue(issue) => Some(issue.project_id),
                api::IssueOrEpic::Epic(_epic) => None,
            })
            .collect();

        let projects = self.get_projects(project_ids)?;

        linked_items
            .into_iter()
            .filter_map(|i| {
                i.to_vcs_service_linked_issue(&projects, &self.group)
                    .transpose()
            })
            .collect::<anyhow::Result<Vec<_>>>()
    }

    #[tracing::instrument(level = "info", skip(self), err)]
    fn get_issue(
        &self,
        project: &str,
        issue_id: u64,
    ) -> anyhow::Result<Option<vcs_service::Issue>> {
        let endpoint = GetIssue::builder()
            .project(NameOrId::Name(project.into()))
            .issue(issue_id)
            .build()
            .context("couldn't build issue endpoint")?;

        let issue: Issue = match endpoint.query(&self.client) {
            Ok(issue) => issue,
            Err(gitlab::api::ApiError::GitlabWithStatus { status, .. })
                if status == http::StatusCode::NOT_FOUND =>
            {
                return Ok(None);
            }
            Err(e) => {
                return Err(anyhow::Error::new(e).context(format!(
                    "couldn't fetch issue {issue_id} in project {project}"
                )));
            }
        };

        let projects = self.get_projects([issue.project_id].into_iter().collect())?;
        let linked_issues = self.get_linked_issues(project, issue.iid)?;

        Ok(Some(issue.to_vcs_service_issue(
            &projects,
            &self.group,
            vcs_service::LinkedIssues::Fetched(linked_issues),
        )?))
    }

    #[tracing::instrument(level = "info", skip(self), err)]
    fn update_issue_description(
        &self,
        project: &str,
        issue_id: u64,
        description: &str,
    ) -> anyhow::Result<()> {
        let endpoint = gitlab::api::projects::issues::EditIssue::builder()
            .project(project)
            .issue(issue_id)
            .description(description)
            .build()
            .context("couldn't build issue editing endpoint")?;

        let _: Issue = endpoint.query(&self.client).with_context(|| {
            format!("couldn't update description for issue {issue_id} in project {project}")
        })?;
        Ok(())
    }

    #[tracing::instrument(level = "info", skip(self), err)]
    fn get_open_issue_with_title(
        &self,
        project: &str,
        title: &str,
    ) -> anyhow::Result<Option<vcs_service::Issue>> {
        let endpoint = gitlab::api::projects::issues::Issues::builder()
            .project(project)
            .state(IssueState::Opened)
            .search(title)
            .search_in(gitlab::api::projects::issues::IssueSearchScope::Title)
            .build()
            .context("couldn't build project issues search endpoint")?;

        let issues: Vec<Issue> = endpoint.query(&self.client).with_context(|| {
            format!("couldn't search open issues in project {project} for title {title:?}")
        })?;

        let Some(issue) = issues.into_iter().find(|i| i.title == title) else {
            return Ok(None);
        };

        let projects = self.get_projects([issue.project_id].into_iter().collect())?;
        let linked_issues = self.get_linked_issues(project, issue.iid)?;
        Ok(Some(issue.to_vcs_service_issue(
            &projects,
            &self.group,
            vcs_service::LinkedIssues::Fetched(linked_issues),
        )?))
    }

    #[tracing::instrument(level = "info", skip(self, description, labels), err)]
    fn create_issue(
        &self,
        project: &str,
        title: &str,
        description: &str,
        labels: &[&str],
    ) -> anyhow::Result<vcs_service::Issue> {
        let endpoint = gitlab::api::projects::issues::CreateIssue::builder()
            .project(project)
            .title(title)
            .description(description)
            .labels(labels.iter().copied())
            .build()
            .context("couldn't build issue creation endpoint")?;

        let issue: Issue = endpoint
            .query(&self.client)
            .with_context(|| format!("couldn't create issue in project {project}"))?;

        let project = self.get_project(issue.project_id)?;
        issue.to_vcs_service_issue(
            &BTreeMap::from_iter([(issue.project_id, project)]),
            &self.group,
            vcs_service::LinkedIssues::Fetched(vec![]),
        )
    }

    #[tracing::instrument(level = "info", skip(self), err)]
    fn create_issue_link(
        &self,
        source_project: &str,
        source_iid: u64,
        target_project: &str,
        target_iid: u64,
        link_type: vcs_service::IssueLinkType,
    ) -> anyhow::Result<()> {
        let endpoint = CreateIssueLink::builder()
            .project(NameOrId::Name(source_project.into()))
            .issue(source_iid)
            .target_project(NameOrId::Name(target_project.into()))
            .target_issue(target_iid)
            .link_type(link_type.into())
            .build()
            .context("couldn't build issue link creation endpoint")?;

        gitlab::api::ignore(endpoint)
            .query(&self.client)
            .with_context(|| {
                format!(
                    "couldn't create issue link {source_project}#{source_iid} -> \
                     {target_project}#{target_iid}"
                )
            })?;
        Ok(())
    }

    #[tracing::instrument(level = "info", skip(self), err)]
    fn get_raw_file(&self, project: &str, path: &str) -> anyhow::Result<Option<String>> {
        let endpoint = gitlab::api::projects::repository::files::FileRaw::builder()
            .project(project)
            .file_path(path)
            .ref_("HEAD")
            .build()
            .context("couldn't build repository file endpoint")?;

        match gitlab::api::raw(endpoint).query(&self.client) {
            Ok(bytes) => {
                let text = String::from_utf8(bytes)
                    .with_context(|| format!("issue template {path} is not valid UTF-8"))?;
                Ok(Some(text))
            }
            Err(gitlab::api::ApiError::GitlabWithStatus { status, .. })
                if status == http::StatusCode::NOT_FOUND =>
            {
                Ok(None)
            }
            Err(e) => Err(anyhow::Error::new(e).context(format!(
                "couldn't fetch issue template {path} from project {project}"
            ))),
        }
    }

    fn project_path_from_url(&self, url: &Url) -> anyhow::Result<String> {
        let path = url.path().trim_matches('/');
        if path.is_empty() {
            bail!("URL {url} has an empty path");
        }

        Ok(path.to_owned())
    }
}

fn issue_state(state: vcs_service::IssueState) -> IssueState {
    match state {
        vcs_service::IssueState::Opened => IssueState::Opened,
        vcs_service::IssueState::Closed => IssueState::Closed,
    }
}

#[derive(Debug, Builder, Clone)]
struct Issues<'a> {
    group: &'a str,

    /// Filter issues based on milestone
    #[builder(default)]
    milestone: Option<&'a str>,

    /// Filter issues based on state
    #[builder(default)]
    state: Option<IssueState>,

    /// Filter issues based on labels; an issue must carry all of them
    #[builder(default)]
    labels: &'a [&'a str],
}

impl<'a> Issues<'a> {
    /// Create a builder for the endpoint.
    pub fn builder(group: &'a str) -> IssuesBuilder<'a> {
        let mut builder = IssuesBuilder::default();
        let _ = builder.group(group);
        builder
    }
}

impl Endpoint for Issues<'_> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("groups/{}/issues", urlencoding::encode(self.group)).into()
    }

    fn parameters(&self) -> QueryParams<'_> {
        let mut params = QueryParams::default();

        let _ = params
            .push_opt("state", self.state)
            .push_opt("milestone", self.milestone);

        if !self.labels.is_empty() {
            let _ = params.push("labels", self.labels.join(","));
        }

        params
    }
}

#[derive(Debug, Builder, Clone)]
struct LinkedItems<'a> {
    #[builder(setter(into))]
    project: NameOrId<'a>,

    issue: u64,
}

impl<'a> LinkedItems<'a> {
    /// Create a builder for the endpoint.
    fn builder() -> LinkedItemsBuilder<'a> {
        LinkedItemsBuilder::default()
    }
}

impl Endpoint for LinkedItems<'_> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/issues/{}/links", self.project, self.issue).into()
    }
}

#[derive(Debug, Builder, Clone)]
struct GetIssue<'a> {
    #[builder(setter(into))]
    project: NameOrId<'a>,

    issue: u64,
}

impl<'a> GetIssue<'a> {
    fn builder() -> GetIssueBuilder<'a> {
        GetIssueBuilder::default()
    }
}

impl Endpoint for GetIssue<'_> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/issues/{}", self.project, self.issue).into()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum CreateIssueLinkType {
    Blocks,
    IsBlockedBy,
    RelatesTo,
}

impl ParamValue<'static> for CreateIssueLinkType {
    fn as_value(&self) -> Cow<'static, str> {
        match self {
            Self::Blocks => "blocks".into(),
            Self::IsBlockedBy => "is_blocked_by".into(),
            Self::RelatesTo => "relates_to".into(),
        }
    }
}

impl From<vcs_service::IssueLinkType> for CreateIssueLinkType {
    fn from(value: vcs_service::IssueLinkType) -> Self {
        match value {
            vcs_service::IssueLinkType::Blocks => Self::Blocks,
            vcs_service::IssueLinkType::IsBlockedBy => Self::IsBlockedBy,
            vcs_service::IssueLinkType::RelatesTo => Self::RelatesTo,
        }
    }
}

/// Endpoint that creates a link between two issues.
///
/// The GitLab API client crate does not expose an issue-links endpoint, so we
/// implement the `POST projects/:id/issues/:iid/links` call ourselves.
#[derive(Debug, Builder, Clone)]
struct CreateIssueLink<'a> {
    #[builder(setter(into))]
    project: NameOrId<'a>,

    issue: u64,

    #[builder(setter(into))]
    target_project: NameOrId<'a>,

    target_issue: u64,

    link_type: CreateIssueLinkType,
}

impl<'a> CreateIssueLink<'a> {
    fn builder() -> CreateIssueLinkBuilder<'a> {
        CreateIssueLinkBuilder::default()
    }
}

impl Endpoint for CreateIssueLink<'_> {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/issues/{}/links", self.project, self.issue).into()
    }

    fn parameters(&self) -> QueryParams<'_> {
        let mut params = QueryParams::default();

        let _ = params
            .push("target_project_id", &self.target_project)
            .push("target_issue_iid", self.target_issue)
            .push("link_type", self.link_type);

        params
    }
}
