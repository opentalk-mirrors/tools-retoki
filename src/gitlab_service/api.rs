use std::collections::BTreeMap;

use gitlab::api::ParamValue;
use jiff::{civil::Date, Timestamp};
use serde::{Deserialize, Serialize};
use snafu::{OptionExt, Whatever};
use url::Url;

use crate::vcs_service;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct Milestone {
    pub id: usize,
    pub iid: usize,
    pub group_id: usize,
    pub title: String,
    pub description: String,
    pub due_date: Option<Date>,
    pub start_date: Option<Date>,
    pub state: MilestoneState,
    pub updated_at: Timestamp,
    pub created_at: Timestamp,
    pub expired: bool,
    pub web_url: Url,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MilestoneState {
    Active,
    Closed,
}

impl MilestoneState {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Closed => "closed",
        }
    }
}

impl ParamValue<'static> for MilestoneState {
    fn as_value(&self) -> std::borrow::Cow<'static, str> {
        self.as_str().into()
    }
}

impl From<Milestone> for vcs_service::Milestone {
    fn from(
        Milestone {
            id,
            title,
            due_date,
            ..
        }: Milestone,
    ) -> Self {
        Self {
            id,
            title,
            due_date,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct Issue {
    pub id: u64,
    pub title: String,
    pub project_id: u64,
}

impl Issue {
    pub(crate) fn to_vcs_service_issue(
        &self,
        projects: &BTreeMap<u64, Project>,
    ) -> Result<vcs_service::Issue, Whatever> {
        let Self {
            id,
            title,
            project_id,
        } = self.clone();
        let project = projects
            .get(&project_id)
            .with_whatever_context(|| {
                format!(
                    "couldn't find project {} in downloaded projects",
                    self.project_id
                )
            })?
            .clone()
            .into();
        Ok(vcs_service::Issue { id, title, project })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct Project {
    pub id: u64,
    pub description: Option<String>,
    pub path: String,
    pub path_with_namespace: String,
    pub namespace: ProjectNamespace,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct ProjectNamespace {
    pub path: String,
}

impl From<Project> for vcs_service::Project {
    fn from(
        Project {
            id,
            path_with_namespace,
            ..
        }: Project,
    ) -> Self {
        Self {
            id,
            path_with_namespace,
        }
    }
}
