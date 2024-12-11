// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

mod api;

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
};

use derive_builder::Builder;
use gitlab::{
    api::{common::NameOrId, issues::IssueState, Endpoint, Query as _, QueryParams},
    Gitlab,
};
use http::Method;
use snafu::{ResultExt, Whatever};
use url::Url;

use self::api::{LinkedIssue, MilestoneState};
use crate::{
    gitlab_service::api::{Issue, Milestone, Project},
    vcs_service::{self, VcsService},
};

pub(crate) struct GitlabService {
    group: String,
    client: Gitlab,
}

impl GitlabService {
    pub(crate) fn connect(url: Url, token: String, group: String) -> Result<Self, Whatever> {
        let host = url
            .as_str()
            .trim_start_matches(&format!("{}://", url.scheme()));
        let client = Gitlab::new(host, token)
            .with_whatever_context(|_e| format!("failed to create gitlab client for {}", host))?;

        Ok(Self { group, client })
    }

    fn get_projects(&self, project_ids: BTreeSet<u64>) -> Result<BTreeMap<u64, Project>, Whatever> {
        let mut projects = BTreeMap::new();
        for project_id in project_ids {
            let endpoint = gitlab::api::projects::Project::builder()
                .project(project_id)
                .build()
                .with_whatever_context(|_| {
                    format!("failed to build endpoint for project {project_id}")
                })?;
            let _ = projects.insert(
                project_id,
                endpoint
                    .query(&self.client)
                    .with_whatever_context(|_| format!("failed to fetch project {project_id}"))?,
            );
        }

        Ok(projects)
    }
}

impl VcsService for GitlabService {
    fn get_milestones(&self) -> Result<Vec<vcs_service::Milestone>, Whatever> {
        let endpoint = GroupMilestones::builder()
            .group(&self.group)
            .state(Some(MilestoneState::Active))
            .build()
            .whatever_context("couldn't build group milestones endpoint")?;

        let milestones: Vec<Milestone> =
            endpoint.query(&self.client).with_whatever_context(|_e| {
                format!("couldn't get milestones for group {}", self.group)
            })?;

        Ok(milestones.into_iter().map(From::from).collect())
    }

    fn get_open_issues_with_milestone_and_label(
        &self,
        milestone: &str,
        label: &str,
    ) -> Result<Vec<vcs_service::Issue>, Whatever> {
        let endpoint = Issues::builder(&self.group)
            .milestone(Some(milestone))
            .label(Some(label))
            .state(Some(IssueState::Opened))
            .build()
            .whatever_context("couldn't build project issues endpoint")?;

        let issues: Vec<Issue> = endpoint.query(&self.client).with_whatever_context(|_e| {
            format!("couldn't get issues for milestone {milestone} with label {label}")
        })?;

        let project_ids = issues.iter().map(|i| i.project_id).collect();

        let projects = self.get_projects(project_ids)?;

        issues
            .into_iter()
            .map(|i| {
                let linked_issues = self.get_linked_issues(&i.project_id.to_string(), i.iid)?;
                i.to_vcs_service_issue(&projects, &self.group, linked_issues)
            })
            .collect::<Result<Vec<_>, Whatever>>()
    }

    fn get_linked_issues(
        &self,
        project: &str,
        issue_id: u64,
    ) -> Result<Vec<vcs_service::LinkedIssue>, Whatever> {
        let endpoint = LinkedIssues::builder()
            .project(NameOrId::Name(project.into()))
            .issue(issue_id)
            .build()
            .whatever_context("couldn't build linked issues endpoint")?;

        let linked_issues: Vec<LinkedIssue> =
            endpoint.query(&self.client).with_whatever_context(|_e| {
                format!("couldn't get linked issues for issue {issue_id} in project {project}")
            })?;

        let project_ids = linked_issues.iter().map(|i| i.issue.project_id).collect();

        let projects = self.get_projects(project_ids)?;

        linked_issues
            .into_iter()
            .map(|i| i.to_vcs_service_linked_issue(&projects, &self.group))
            .collect::<Result<Vec<_>, Whatever>>()
    }

    fn update_issue_description(
        &self,
        project: &str,
        issue_id: u64,
        description: &str,
    ) -> Result<(), Whatever> {
        let endpoint = gitlab::api::projects::issues::EditIssue::builder()
            .project(project)
            .issue(issue_id)
            .description(description)
            .build()
            .whatever_context("couldn't build issue editing endpoint")?;

        let _: Issue = endpoint.query(&self.client).with_whatever_context(|_e| {
            format!("couldn't update description for issue {issue_id} in project {project}")
        })?;
        Ok(())
    }
}

#[derive(Debug, Builder, Clone)]
struct GroupMilestones<'a> {
    #[builder(setter(into))]
    group: NameOrId<'a>,

    /// Filter milestones based on state
    #[builder(default)]
    state: Option<MilestoneState>,
}

impl<'a> GroupMilestones<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> GroupMilestonesBuilder<'a> {
        GroupMilestonesBuilder::default()
    }
}

impl Endpoint for GroupMilestones<'_> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("groups/{}/milestones", self.group).into()
    }

    fn parameters(&self) -> QueryParams {
        let mut params = QueryParams::default();

        let _ = params.push_opt("state", self.state);

        params
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

    fn parameters(&self) -> QueryParams {
        let mut params = QueryParams::default();

        let _ = params
            .push_opt("state", self.state)
            .push_opt("milestone", self.milestone)
            .push_opt("labels", self.label);

        params
    }
}

#[derive(Debug, Builder, Clone)]
struct LinkedIssues<'a> {
    #[builder(setter(into))]
    project: NameOrId<'a>,

    issue: u64,
}

impl<'a> LinkedIssues<'a> {
    /// Create a builder for the endpoint.
    fn builder() -> LinkedIssuesBuilder<'a> {
        LinkedIssuesBuilder::default()
    }
}

impl Endpoint for LinkedIssues<'_> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/issues/{}/links", self.project, self.issue).into()
    }
}
