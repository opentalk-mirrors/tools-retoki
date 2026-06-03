// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

mod api;

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
};

use anyhow::Context as _;
use derive_builder::Builder;
use gitlab::{
    api::{common::NameOrId, issues::IssueState, Endpoint, Query as _, QueryParams},
    Gitlab,
};
use http::Method;
use rayon::iter::{IntoParallelIterator, ParallelIterator as _};
use url::Url;

use self::api::LinkedItem;
use crate::{
    gitlab_service::api::{Issue, Project},
    vcs_service::{self, VcsService},
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
            .map(|project_id| {
                let endpoint = gitlab::api::projects::Project::builder()
                    .project(project_id)
                    .build()
                    .with_context(|| {
                        format!("failed to build endpoint for project {project_id}")
                    })?;
                let project: Project = endpoint
                    .query(&self.client)
                    .with_context(|| format!("failed to fetch project {project_id}"))?;
                Ok((project_id, project))
            })
            .collect()
    }
}

impl VcsService for GitlabService {
    #[tracing::instrument(level = "info", skip(self), err)]
    fn get_open_issues_with_label(&self, label: &str) -> anyhow::Result<Vec<vcs_service::Issue>> {
        let endpoint = Issues::builder(&self.group)
            .label(Some(label))
            .state(Some(IssueState::Opened))
            .build()
            .context("couldn't build project issues endpoint")?;

        let issues: Vec<Issue> = endpoint
            .query(&self.client)
            .with_context(|| format!("couldn't get issues with label {label}"))?;

        let project_ids = issues.iter().map(|i| i.project_id).collect();

        let projects = self.get_projects(project_ids)?;

        issues
            .into_par_iter()
            .map(|i| {
                let linked_issues = self.get_linked_issues(&i.project_id.to_string(), i.iid)?;
                i.to_vcs_service_issue(&projects, &self.group, linked_issues)
            })
            .collect::<anyhow::Result<Vec<_>>>()
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

    /// Filter issues based on label
    #[builder(default)]
    label: Option<&'a str>,
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
            .push_opt("milestone", self.milestone)
            .push_opt("labels", self.label);

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
