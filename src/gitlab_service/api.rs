// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use anyhow::Context as _;
use serde::{Deserialize, Serialize};

use crate::vcs_service;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct Issue {
    pub id: u64,
    pub iid: u64,
    pub title: String,
    pub description: Option<String>,
    pub project_id: u64,
    pub references: IssueReferences,
    pub state: IssueState,
}

impl Issue {
    pub(crate) fn to_vcs_service_issue(
        &self,
        projects: &BTreeMap<u64, Project>,
        gitlab_group: &str,
        linked_issues: Vec<vcs_service::LinkedIssue>,
    ) -> anyhow::Result<vcs_service::Issue> {
        let Self {
            id,
            iid,
            title,
            description,
            project_id,
            references,
            state,
        } = self.clone();
        let project = projects
            .get(&project_id)
            .with_context(|| {
                format!(
                    "couldn't find project {} in downloaded projects",
                    self.project_id
                )
            })?
            .clone()
            .into();
        Ok(vcs_service::Issue {
            id,
            iid,
            title,
            description,
            project,
            short_reference: references
                .full
                .trim_start_matches(&format!("{}/", gitlab_group))
                .to_string(),
            state: state.into(),
            linked_issues,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct Epic {
    pub id: u64,
    pub iid: u64,
    pub title: String,
    pub description: Option<String>,
    pub references: IssueReferences,
    pub state: IssueState,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum IssueOrEpic {
    Issue(Issue),
    Epic(Epic),
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct IssueReferences {
    pub full: String,
    pub relative: String,
    pub short: String,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IssueLinkType {
    Blocks,
    IsBlockedBy,
    RelatesTo,
}

impl From<IssueLinkType> for vcs_service::IssueLinkType {
    fn from(value: IssueLinkType) -> Self {
        match value {
            IssueLinkType::Blocks => Self::Blocks,
            IssueLinkType::IsBlockedBy => Self::IsBlockedBy,
            IssueLinkType::RelatesTo => Self::RelatesTo,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct LinkedItem {
    pub link_type: IssueLinkType,

    #[serde(flatten)]
    pub item: IssueOrEpic,
}

impl LinkedItem {
    pub(crate) fn to_vcs_service_linked_issue(
        &self,
        projects: &BTreeMap<u64, Project>,
        gitlab_group: &str,
    ) -> anyhow::Result<Option<vcs_service::LinkedIssue>> {
        let Self {
            link_type,
            item: IssueOrEpic::Issue(issue),
        } = self.clone()
        else {
            return Ok(None);
        };

        let issue = issue.to_vcs_service_issue(projects, gitlab_group, vec![])?;

        Ok(Some(vcs_service::LinkedIssue {
            link_type: link_type.into(),
            issue,
        }))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IssueState {
    Opened,
    Closed,
}

impl From<IssueState> for vcs_service::IssueState {
    fn from(value: IssueState) -> Self {
        match value {
            IssueState::Opened => Self::Opened,
            IssueState::Closed => Self::Closed,
        }
    }
}
