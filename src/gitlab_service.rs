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

use crate::{
    gitlab_service::api::{Issue, Milestone, Project},
    vcs_service::VcsService,
};

use self::api::MilestoneState;

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
    fn get_milestones(&self) -> Result<Vec<crate::vcs_service::Milestone>, Whatever> {
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
    ) -> Result<Vec<crate::vcs_service::Issue>, Whatever> {
        let endpoint = Issues::builder()
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
            .map(|i| i.to_vcs_service_issue(&projects))
            .collect::<Result<Vec<_>, Whatever>>()
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

impl<'a> Endpoint for GroupMilestones<'a> {
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
    pub fn builder() -> IssuesBuilder<'a> {
        IssuesBuilder::default()
    }
}

impl<'a> Endpoint for Issues<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        "issues".into()
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
