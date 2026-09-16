// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{ComponentIdentifier, GroupIdentifier};

/// A profile allows to specify additional, context dependent information for
/// [`crate::data::Releases`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "ProfileRaw")]
pub struct Profile {
    pub profile_name: String,
    #[serde(rename = "components")]
    pub standalone_components: IndexMap<ComponentIdentifier, ComponentProfile>,
    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    pub groups: IndexMap<GroupIdentifier, ComponentGroup>,
}

// Serde entry point for `Profile`. Kept private and only used via `#[serde(try_from = ...)]`
// so validation runs on every deserialization path.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileRaw {
    profile_name: String,
    #[serde(default)]
    components: IndexMap<ComponentIdentifier, ComponentProfile>,
    #[serde(default)]
    groups: IndexMap<GroupIdentifier, ComponentGroup>,
}

/// Errors produced when validating a [`Profile`] after deserialization.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProfileValidationError {
    #[error("group `{0}` has no components")]
    EmptyGroup(GroupIdentifier),

    #[error(
        "component `{component}` appears in groups `{first_group}` and `{second_group}`; \
         each component may belong to at most one group"
    )]
    DuplicateGroupMember {
        component: ComponentIdentifier,
        first_group: GroupIdentifier,
        second_group: GroupIdentifier,
    },

    #[error("component `{component}` appears in `components` and group `{group}`")]
    StandaloneAndGroupMember {
        component: ComponentIdentifier,
        group: GroupIdentifier,
    },
}

impl TryFrom<ProfileRaw> for Profile {
    type Error = ProfileValidationError;

    fn try_from(raw: ProfileRaw) -> Result<Self, Self::Error> {
        for (group_id, group) in &raw.groups {
            if group.components.is_empty() {
                return Err(ProfileValidationError::EmptyGroup(group_id.clone()));
            }
        }

        let mut seen: IndexMap<&ComponentIdentifier, &GroupIdentifier> = IndexMap::new();
        for (group_id, group) in &raw.groups {
            for component_id in group.components.keys() {
                // Components are not allowed to be present in both the `components` section and a
                // group.
                if raw.components.contains_key(component_id) {
                    return Err(ProfileValidationError::StandaloneAndGroupMember {
                        component: component_id.clone(),
                        group: group_id.clone(),
                    });
                }

                // Components are not allowed to be present in more than one group.
                if let Some(first_group) = seen.insert(component_id, group_id) {
                    return Err(ProfileValidationError::DuplicateGroupMember {
                        component: component_id.clone(),
                        first_group: first_group.clone(),
                        second_group: group_id.clone(),
                    });
                }
            }
        }

        Ok(Profile {
            profile_name: raw.profile_name,
            standalone_components: raw.components,
            groups: raw.groups,
        })
    }
}

impl Profile {
    /// Look up the effective profile entry for `id`.
    ///
    /// A standalone entry shadows a group member with the same identifier, matching the
    /// precedence users would intuit from the file layout.
    pub fn component(&self, id: &ComponentIdentifier) -> Option<ComponentProfileRef<'_>> {
        if let Some(component) = self.standalone_components.get(id) {
            return Some(component.into());
        }

        for group in self.groups.values() {
            if let Some(component) = group.components.get(id) {
                return Some(ComponentProfileRef {
                    gitlab_url: group.gitlab_url.as_deref(),
                    component,
                });
            }
        }

        None
    }

    pub fn all_components(
        &self,
    ) -> impl Iterator<Item = (&ComponentIdentifier, ComponentProfileRef<'_>)> {
        self.standalone_components
            .iter()
            .map(|(id, component)| (id, component.into()))
            .chain(self.groups.values().flat_map(|group| {
                group.components.iter().map(|(id, component)| {
                    (
                        id,
                        ComponentProfileRef {
                            gitlab_url: group.gitlab_url.as_deref(),
                            component,
                        },
                    )
                })
            }))
    }
}

/// A borrowed view of a profile entry, whether it lives standalone or inside a group.
///
/// Returned by [`Profile::component`] so callers get the effective `gitlab_url` (which
/// for a group member is inherited from the group) alongside the component's own
/// configuration without any cloning.
#[derive(Debug, Clone, Copy)]
pub struct ComponentProfileRef<'a> {
    pub gitlab_url: Option<&'a str>,
    pub component: &'a ComponentConfig,
}

impl<'a> From<&'a ComponentProfile> for ComponentProfileRef<'a> {
    fn from(value: &'a ComponentProfile) -> Self {
        Self {
            gitlab_url: value.gitlab_url.as_deref(),
            component: &value.component,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gitlab_url: Option<String>,

    #[serde(flatten)]
    pub component: ComponentConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentGroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gitlab_url: Option<String>,

    /// The name of the group that will be used when creating issues
    pub name: String,

    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    pub components: IndexMap<ComponentIdentifier, ComponentConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_base_url: Option<String>,

    /// List of component identifiers that must be released before this component
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_by: Option<Vec<ComponentIdentifier>>,

    /// The component will be excluded from any documentation
    #[serde(default)]
    pub private: bool,
}

#[cfg(test)]
mod tests {

    use super::*;

    fn group(components: IndexMap<ComponentIdentifier, ComponentConfig>) -> ComponentGroup {
        ComponentGroup {
            gitlab_url: None,
            name: "Group Name".to_owned(),
            components,
        }
    }

    fn component_config() -> ComponentConfig {
        ComponentConfig {
            container_base_url: None,
            blocked_by: None,
            private: false,
        }
    }

    #[test]
    fn empty_group_is_rejected() {
        let group_id = GroupIdentifier::from("empty".to_owned());
        let raw = ProfileRaw {
            profile_name: "Empty Group".to_owned(),
            components: IndexMap::new(),
            groups: IndexMap::from_iter([(group_id.clone(), group(IndexMap::new()))]),
        };
        let error = Profile::try_from(raw).unwrap_err();
        assert_eq!(error, ProfileValidationError::EmptyGroup(group_id));
    }

    #[test]
    fn duplicate_group_member_is_rejected() {
        let controller_id = ComponentIdentifier::from("controller".to_owned());
        let first_group = GroupIdentifier::from("first".to_owned());
        let second_group = GroupIdentifier::from("second".to_owned());
        let raw = ProfileRaw {
            profile_name: "Duplicate Group Member".to_owned(),
            components: IndexMap::new(),
            groups: IndexMap::from_iter([
                (
                    first_group.clone(),
                    group(IndexMap::from_iter([(
                        controller_id.clone(),
                        component_config(),
                    )])),
                ),
                (
                    second_group.clone(),
                    group(IndexMap::from_iter([(
                        controller_id.clone(),
                        component_config(),
                    )])),
                ),
            ]),
        };

        let error = Profile::try_from(raw).unwrap_err();
        assert_eq!(
            error,
            ProfileValidationError::DuplicateGroupMember {
                component: controller_id,
                first_group,
                second_group,
            }
        );
    }

    #[test]
    fn standalone_and_group_member_is_rejected() {
        let controller_id = ComponentIdentifier::from("controller".to_owned());
        let group_id = GroupIdentifier::from("group".to_owned());
        let raw = ProfileRaw {
            profile_name: "Standalone and Group Member".to_owned(),
            components: IndexMap::from_iter([(
                controller_id.clone(),
                ComponentProfile {
                    gitlab_url: None,
                    component: component_config(),
                },
            )]),
            groups: IndexMap::from_iter([(
                group_id.clone(),
                group(IndexMap::from_iter([(
                    controller_id.clone(),
                    component_config(),
                )])),
            )]),
        };

        let error = Profile::try_from(raw).unwrap_err();
        assert_eq!(
            error,
            ProfileValidationError::StandaloneAndGroupMember {
                component: controller_id,
                group: group_id,
            }
        );
    }
}
