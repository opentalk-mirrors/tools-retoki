// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::path::{Path, PathBuf};

use anyhow::Context;
use indexmap::IndexMap;
use semver::Version;
use time::{Date, OffsetDateTime};

use crate::{
    bot_templates::{CategoryData, ComponentData},
    data::{
        Component, ComponentCategoryName, ComponentIdentifier, ComponentName, ComponentVersion,
        Profile, Release, ReleaseSeries, SeriesNumber, read_profile_file, read_release_file,
        write_releases_file,
    },
    vcs_service::{Issue, IssueLinkType, IssueState, LinkedIssue, VcsService},
};

#[derive(Debug)]
pub(crate) struct ReleasesBuilder {
    dry_run: bool,
}

impl ReleasesBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self { dry_run: false }
    }

    #[must_use]
    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    pub fn load(self, path: PathBuf, profile_path: &Path) -> anyhow::Result<Releases> {
        let releases = read_release_file(&path)
            .with_context(|| format!("Failed to read release file at {}", path.display()))?;
        let profile = read_profile_file(profile_path)
            .with_context(|| format!("Failed to read profile {}", profile_path.display()))?;

        Ok(Releases {
            releases,
            profile,
            path,
            dry_run: self.dry_run,
        })
    }
}

pub(crate) struct ReleasesEditor<'a> {
    inner: &'a mut Releases,
}

impl ReleasesEditor<'_> {
    pub fn get_or_insert_from_previous(&mut self, version: Version) -> anyhow::Result<Release> {
        self.inner.get_or_insert_from_previous(version)
    }

    /// Set the planned `component_version` of `component` in the release entry
    /// for `version`, returning the updated release.
    ///
    /// Fails if no release entry exists for `version`; callers are expected to
    /// have created it via `init` beforehand.
    pub fn set_component_version(
        &mut self,
        version: &Version,
        component: ComponentIdentifier,
        component_version: ComponentVersion,
    ) -> anyhow::Result<Release> {
        self.inner
            .set_component_version(version, component, component_version)
    }
}

#[must_use = "edits are not persisted until you call `.save()` (use `.discard()` to drop them)"]
pub(crate) struct Staged<'a, T> {
    releases: &'a mut Releases,
    value: T,
}

/// A component resolved against both the releases data and the loaded profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedComponent {
    pub id: ComponentIdentifier,
    pub name: ComponentName,
    pub category_name: String,
    pub gitlab_url: Option<String>,
}

impl<T> Staged<'_, T> {
    /// Persist the changes and return the closure's value.
    pub fn save(self) -> anyhow::Result<T> {
        self.releases.save()?;
        Ok(self.value)
    }

    /// Keep the changes in memory only.
    #[expect(unused)]
    pub fn discard(self) -> T {
        self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Releases {
    releases: crate::data::Releases,
    pub profile: Profile,
    path: PathBuf,
    dry_run: bool,
}
impl Releases {
    pub fn edit<T>(
        &mut self,
        f: impl FnOnce(&mut ReleasesEditor) -> anyhow::Result<T>,
    ) -> anyhow::Result<Staged<'_, T>> {
        let value = f(&mut ReleasesEditor { inner: self })?;

        Ok(Staged {
            releases: self,
            value,
        })
    }

    /// Return the highest released version less than `version`.
    /// Pre-releases are included, so seeding a stable release after a beta
    /// picks up the beta's components.
    #[must_use]
    pub fn previous_release(&self, version: &Version) -> Option<&Release> {
        self.releases.previous_release(version)
    }

    /// Insert a new release entry for `version` based on the previous release's components.
    fn insert_new_from_previous(&mut self, version: Version) -> &Release {
        let components = self.previous_release(&version).map(|release| release.components.clone()).unwrap_or_else(|| {
           tracing::warn!("No previous release exists before {version}. You'll have to add the components manually.");
           Default::default()
        });
        let series_nr = SeriesNumber::from(&version);
        let today = OffsetDateTime::now_utc().date();
        let series = self
            .releases
            .series
            .entry(series_nr)
            .or_insert_with(|| ReleaseSeries {
                end_of_life: end_of_month(today),
                releases: Default::default(),
            });

        series
            .releases
            .entry(version)
            .insert_entry(Release {
                date: today,
                components,
                release_notes: None,
                tickets: Vec::new(),
            })
            .into_mut()
    }

    fn get_or_insert_from_previous(&mut self, version: Version) -> anyhow::Result<Release> {
        let series_nr = SeriesNumber::from(&version);

        if let Some(release) = self
            .releases
            .series
            .get(&series_nr)
            .and_then(|s| s.releases.get(&version))
        {
            return Ok(release.clone());
        }

        Ok(self.insert_new_from_previous(version).clone())
    }

    /// Set the planned `component_version` of `component` in the release entry
    /// for `version`, returning the updated release. Fails if no such release
    /// entry exists.
    fn set_component_version(
        &mut self,
        version: &Version,
        component: ComponentIdentifier,
        component_version: ComponentVersion,
    ) -> anyhow::Result<Release> {
        let series_nr = SeriesNumber::from(version);
        let release = self
            .releases
            .series
            .get_mut(&series_nr)
            .and_then(|series| series.releases.get_mut(version))
            .with_context(|| {
                format!(
                    "no release entry for {version} in releases.yml; \
                     run `retoki release {version} init` first"
                )
            })?;
        let _ = release.components.insert(component, component_version);
        Ok(release.clone())
    }

    /// Look up `component` in the releases data and the loaded profile,
    /// returning the metadata that the release workflow commonly needs.
    ///
    /// Fails if the component is not declared in `releases.yml`. A missing
    /// profile entry is not an error: it simply yields no `gitlab_url`, which
    /// marks the component as one without its own release project (e.g. a
    /// third-party component).
    pub fn resolve_component(&self, component: &str) -> anyhow::Result<ResolvedComponent> {
        let id = ComponentIdentifier::from(component.to_owned());
        let component =
            self.releases.components.get(&id).with_context(|| {
                format!("component {component} is not declared in releases.yml")
            })?;
        let category_name = self
            .releases
            .component_categories
            .get(&component.category)
            .map(|category| category.name.to_string())
            .unwrap_or_else(|| component.category.to_string());
        let gitlab_url = self
            .profile
            .component(&id)
            .and_then(|profile| profile.gitlab_url)
            .map(str::to_owned);
        Ok(ResolvedComponent {
            id,
            name: component.name.clone(),
            category_name,
            gitlab_url,
        })
    }

    /// Names of already-released components whose release would be invalidated
    /// by (re)adding `component`.
    ///
    /// A component `b` blocks updates to `component` when `b`'s profile lists
    /// `component` in its `blocked_by` (i.e. `component` must be released before
    /// `b`). If such a `b` has already been released, `component` should not be
    /// changed since this would invalidate the already existing release of `b`.
    pub fn released_blocking_components(
        &self,
        component: &ComponentIdentifier,
        product_release_issue: &Issue,
        vcs: &dyn VcsService,
    ) -> anyhow::Result<Vec<ComponentName>> {
        let mut blocking = Vec::new();

        for (id, profile) in self.profile.all_components() {
            let is_blocking = profile
                .component
                .blocked_by
                .as_deref()
                .is_some_and(|blocked_by| blocked_by.contains(component));
            if !is_blocking {
                continue;
            }

            let name = self
                .releases
                .components
                .get(id)
                .map(|component| component.name.clone())
                .unwrap_or_else(|| ComponentName::from(id.to_string()));

            let Some(gitlab_url) = profile.gitlab_url else {
                tracing::warn!("Skipping `{name}` as it does not provide a GitLab URL");
                continue;
            };
            let url = gitlab_url
                .parse()
                .with_context(|| format!("invalid gitlab_url {gitlab_url:?} for component {id}"))?;
            let project_path = vcs.project_path_from_url(&url).with_context(|| {
                format!("couldn't derive project path from gitlab_url {gitlab_url:?}")
            })?;

            let released = product_release_issue.linked_issues.iter().any(|linked| {
                linked.link_type == IssueLinkType::IsBlockedBy
                    && linked.issue.project.path_with_namespace == project_path
                    && linked.issue.state == IssueState::Closed
            });
            if released {
                blocking.push(name);
            }
        }

        Ok(blocking)
    }

    /// Return the planned version of `component` in the release immediately
    /// preceding `version`, if that release exists and carries a semver version
    /// for the component.
    #[must_use]
    pub fn previous_component_version(
        &self,
        version: &Version,
        component: &ComponentIdentifier,
    ) -> Option<Version> {
        self.previous_release(version)
            .and_then(|previous| previous.components.get(component))
            .and_then(|version| version.as_semver().cloned())
    }

    pub fn category_data(
        &self,
        version: &Version,
        release: &Release,
        linked_issues: &[LinkedIssue],
        vcs: &dyn VcsService,
    ) -> anyhow::Result<impl Iterator<Item = CategoryData>> {
        let previous_release = self.previous_release(version);
        let mut categories = IndexMap::new();

        for (component_id, component_version) in &release.components {
            let Component {
                name: component_name,
                category,
                ..
            } = self
                .releases
                .components
                .get(component_id)
                .with_context(|| {
                    format!("component {component_id} is not declared in releases.yml")
                })?;
            let category = category.clone();

            let data = build_component_data(
                component_id,
                component_name.clone(),
                component_version.clone(),
                self.profile
                    .component(component_id)
                    .and_then(|profile| profile.gitlab_url)
                    .map(str::to_owned),
                previous_release,
                linked_issues,
                vcs,
            );
            categories
                .entry(category.clone())
                .or_insert_with(|| {
                    let category_name = self
                        .releases
                        .component_categories
                        .get(&category)
                        .map(|c| c.name.clone())
                        .unwrap_or_else(|| {
                            ComponentCategoryName::from("Unkown category".to_string())
                        });
                    CategoryData {
                        name: category_name,
                        components: Vec::new(),
                    }
                })
                .components
                .push(data);
        }

        Ok(categories.into_values())
    }

    fn save(&self) -> anyhow::Result<()> {
        if self.dry_run {
            return Ok(());
        }

        write_releases_file(&self.path, self.releases.clone())
            .with_context(|| format!("couldn't write releases file at {}", self.path.display()))
    }
}

fn build_component_data(
    id: &ComponentIdentifier,
    name: ComponentName,
    version: ComponentVersion,
    gitlab_url: Option<String>,
    previous_release: Option<&Release>,
    linked_issues: &[LinkedIssue],
    vcs: &dyn VcsService,
) -> ComponentData {
    let prefixed_version = version.prefixed();
    let ticket_url = resolve_ticket_url(&name, &version, gitlab_url.as_ref(), linked_issues, vcs);
    let has_changed = has_component_changed(id, &version, previous_release);

    ComponentData {
        name,
        version,
        prefixed_version,
        has_changed,
        gitlab_url,
        ticket_url,
    }
}

fn has_component_changed(
    id: &ComponentIdentifier,
    version: &ComponentVersion,
    previous_release: Option<&Release>,
) -> bool {
    let Some(previous_release) = previous_release else {
        return false;
    };

    previous_release
        .components
        .get(id)
        .is_none_or(|previous_version| previous_version != version)
}

fn resolve_ticket_url(
    component_name: &ComponentName,
    component_version: &ComponentVersion,
    gitlab_url: Option<&String>,
    linked_issues: &[LinkedIssue],
    vcs: &dyn VcsService,
) -> Option<String> {
    let raw = gitlab_url?;
    let url = raw
        .parse()
        .inspect_err(|err| {
            tracing::error!(%err, %raw, "Failed to parse project URL");
        })
        .ok()?;
    let project_path = vcs
        .project_path_from_url(&url)
        .inspect_err(|err| {
            tracing::error!(%err, %url, "Failed to get project path from URL");
        })
        .ok()?;

    find_release_issue(
        component_name,
        component_version,
        &project_path,
        linked_issues,
    )
    .map(|issue| issue.web_url.to_string())
}

fn find_release_issue<'a>(
    component_name: &ComponentName,
    version: &ComponentVersion,
    project_path: &str,
    linked_issues: &'a [LinkedIssue],
) -> Option<&'a Issue> {
    // Trying to find the release issue by title isn't ideal, but this reflects our current workflow
    // and doesn't require additional metadata (that we currently don't have).
    let expected_title = build_release_title(&component_name.to_string(), version);
    let candidates = linked_issues
        .iter()
        .filter(|linked| linked.link_type == IssueLinkType::IsBlockedBy)
        .map(|linked| &linked.issue)
        .filter(|issue| issue.project.path_with_namespace == project_path);

    Issue::match_release_issue(candidates, &expected_title, &version.to_string())
}

/// Returns the last day of the current calendar month.
fn end_of_month(date: Date) -> Date {
    date.replace_day(date.month().length(date.year()))
        .expect("the last day of the current month is always a valid date")
}

pub fn build_release_title(name: &str, version: &ComponentVersion) -> String {
    format!("Release {} of {}", version.prefixed(), name)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use pretty_assertions::assert_eq;
    use semver::{BuildMetadata, Prerelease};
    use time::Month;
    use url::Url;

    use super::*;
    use crate::vcs_service::{IssueState, Project};

    fn empty_profile() -> Profile {
        Profile {
            profile_name: "test".to_owned(),
            standalone_components: IndexMap::new(),
            groups: IndexMap::new(),
        }
    }

    fn linked_issue(
        title: String,
        project_path: String,
        web_url: Url,
        link_type: IssueLinkType,
    ) -> LinkedIssue {
        LinkedIssue {
            link_type,
            issue: Issue {
                id: 1,
                iid: 1,
                title,
                project: Project {
                    id: 1,
                    path_with_namespace: project_path.to_owned(),
                },
                short_reference: format!("{project_path}#1"),
                description: None,
                state: IssueState::Opened,
                linked_issues: Vec::new(),
                web_url,
            },
        }
    }

    #[test]
    fn end_of_month_handles_varying_month_lengths() {
        assert_eq!(
            end_of_month(Date::from_calendar_date(2023, Month::February, 15).expect("valid date")),
            Date::from_calendar_date(2023, Month::February, 28).expect("valid date"),
        );
        assert_eq!(
            end_of_month(Date::from_calendar_date(2024, Month::February, 10).expect("valid date")),
            Date::from_calendar_date(2024, Month::February, 29).expect("valid date"),
            "February in a leap year has 29 days",
        );
        assert_eq!(
            end_of_month(Date::from_calendar_date(2023, Month::April, 1).expect("valid date")),
            Date::from_calendar_date(2023, Month::April, 30).expect("valid date"),
        );
        assert_eq!(
            end_of_month(Date::from_calendar_date(2023, Month::December, 31).expect("valid date")),
            Date::from_calendar_date(2023, Month::December, 31).expect("valid date"),
        );
    }

    #[test]
    fn build_release_title_formats_version_and_component() {
        assert_eq!(
            build_release_title(
                "controller",
                &ComponentVersion::Semver(Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::EMPTY,
                    build: BuildMetadata::EMPTY
                })
            ),
            "Release v1.2.3 of controller",
        );
    }

    #[test]
    fn has_component_changed_is_false_without_previous_release() {
        assert!(!has_component_changed(
            &ComponentIdentifier::from("a".to_owned()),
            &ComponentVersion::Semver("1.0.0".parse().expect("valid semver")),
            None,
        ));
    }

    #[test]
    fn has_component_changed_detects_changed_and_added_components() {
        let previous = Release {
            date: Date::from_calendar_date(2024, Month::January, 1).expect("valid date"),
            components: IndexMap::from([(
                ComponentIdentifier::from("a".to_owned()),
                ComponentVersion::Semver("1.0.0".parse().expect("valid semver")),
            )]),
            release_notes: None,
            tickets: Vec::new(),
        };

        assert!(
            !has_component_changed(
                &ComponentIdentifier::from("a".to_owned()),
                &ComponentVersion::Semver("1.0.0".parse().expect("valid semver")),
                Some(&previous),
            ),
            "identical version is unchanged",
        );
        assert!(
            has_component_changed(
                &ComponentIdentifier::from("a".to_owned()),
                &ComponentVersion::Semver("2.0.0".parse().expect("valid semver")),
                Some(&previous),
            ),
            "different version is changed",
        );
        assert!(
            has_component_changed(
                &ComponentIdentifier::from("b".to_owned()),
                &ComponentVersion::Semver("1.0.0".parse().expect("valid semver")),
                Some(&previous),
            ),
            "component absent from the previous release counts as changed",
        );
    }

    #[test]
    fn find_release_issue_prefers_exact_title_match() {
        let component = ComponentName::from("controller".to_string());
        let version = ComponentVersion::Semver(Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: Prerelease::EMPTY,
            build: BuildMetadata::EMPTY,
        });
        let mut title = build_release_title(&component.to_string(), &version);

        let project_path = "group/controller".to_owned();
        let url: Url = "https://example.com/exact-issue"
            .parse()
            .expect("valid url");
        let link_type = IssueLinkType::IsBlockedBy;
        let exact = linked_issue(title.clone(), project_path.clone(), url.clone(), link_type);

        title.push_str(" and other things");
        let fuzzy = linked_issue(title, project_path, url, link_type);

        let expected = exact.issue.clone();

        let linked_issues = [exact, fuzzy];
        let found =
            find_release_issue(&component, &version, "group/controller", &linked_issues).unwrap();

        assert_eq!(expected, *found);
    }

    #[test]
    fn find_release_issue_falls_back_to_version_substring() {
        let component = ComponentName::from("controller".to_string());
        let version = ComponentVersion::Semver(Version {
            major: 1,
            minor: 0,
            patch: 0,
            pre: Prerelease::EMPTY,
            build: BuildMetadata::EMPTY,
        });
        let issues = vec![linked_issue(
            "Bump to v1.0.0 please".to_owned(),
            "group/controller".to_owned(),
            "https://example.com/".parse().expect("valid url"),
            IssueLinkType::IsBlockedBy,
        )];

        let found = find_release_issue(&component, &version, "group/controller", &issues);

        assert_eq!(
            found.map(|i| i.web_url.as_str()),
            Some("https://example.com/"),
        );
    }

    #[test]
    fn previous_release() {
        let releases = Releases {
            releases: crate::data::Releases {
                product_name: "OpenTalk".to_owned().into(),
                releases_page_header: None,
                series: BTreeMap::from_iter([(
                    "1.0".parse().expect("valid series"),
                    ReleaseSeries {
                        end_of_life: Date::from_calendar_date(2030, Month::December, 31)
                            .expect("valid date"),
                        releases: IndexMap::from([
                            (
                                "1.0.0".parse().expect("valid semver"),
                                Release {
                                    date: Date::from_calendar_date(2024, Month::January, 1)
                                        .expect("valid date"),
                                    components: IndexMap::from_iter([(
                                        ComponentIdentifier::from("a".to_owned()),
                                        ComponentVersion::Semver(
                                            "1.0.0".parse().expect("valid semver"),
                                        ),
                                    )]),
                                    release_notes: None,
                                    tickets: Vec::new(),
                                },
                            ),
                            (
                                "1.0.1".parse().expect("valid semver"),
                                Release {
                                    date: Date::from_calendar_date(2024, Month::January, 1)
                                        .expect("valid date"),
                                    components: IndexMap::from_iter([(
                                        ComponentIdentifier::from("a".to_owned()),
                                        ComponentVersion::Semver(
                                            "1.0.1".parse().expect("valid semver"),
                                        ),
                                    )]),
                                    release_notes: None,
                                    tickets: Vec::new(),
                                },
                            ),
                            (
                                "1.0.2-beta.1".parse().expect("valid semver"),
                                Release {
                                    date: Date::from_calendar_date(2024, Month::January, 1)
                                        .expect("valid date"),
                                    components: IndexMap::from_iter([(
                                        ComponentIdentifier::from("a".to_owned()),
                                        ComponentVersion::Semver(
                                            "1.0.2-beta.1".parse().expect("valid semver"),
                                        ),
                                    )]),
                                    release_notes: None,
                                    tickets: Vec::new(),
                                },
                            ),
                        ]),
                    },
                )]),
                components: IndexMap::new(),
                component_categories: IndexMap::new(),
            },
            profile: empty_profile(),
            path: PathBuf::new(),
            dry_run: true,
        };

        let previous = releases.previous_release(&"1.0.1".parse().expect("valid semver"));
        assert_eq!(
            previous,
            releases
                .releases
                .get_release(&"1.0.0".parse().expect("valid semver")),
        );

        // Previous release includes beta
        let previous = releases.previous_release(&"1.0.2".parse().expect("valid semver"));
        assert_eq!(
            previous,
            releases
                .releases
                .get_release(&"1.0.2-beta.1".parse().expect("valid semver"))
        );

        // Previous release is none when nothing is lower
        let previous = releases.previous_release(&"1.0.0".parse().expect("valid semver"));
        assert!(previous.is_none());
    }

    #[test]
    fn get_or_insert_returns_existing_without_duplicating() {
        let mut releases = Releases {
            releases: crate::data::Releases {
                product_name: "OpenTalk".to_owned().into(),
                releases_page_header: None,
                series: BTreeMap::from([(
                    "1.0".parse().expect("valid series"),
                    ReleaseSeries {
                        end_of_life: Date::from_calendar_date(2030, Month::December, 31)
                            .expect("valid date"),
                        releases: IndexMap::from([
                            (
                                "1.0.0".parse().expect("valid semver"),
                                Release {
                                    date: Date::from_calendar_date(2024, Month::January, 1)
                                        .expect("valid date"),
                                    components: IndexMap::new(),
                                    release_notes: None,
                                    tickets: Vec::new(),
                                },
                            ),
                            (
                                "1.0.1".parse().expect("valid semver"),
                                Release {
                                    date: Date::from_calendar_date(2024, Month::January, 1)
                                        .expect("valid date"),
                                    components: IndexMap::new(),
                                    release_notes: None,
                                    tickets: Vec::new(),
                                },
                            ),
                        ]),
                    },
                )]),
                components: IndexMap::new(),
                component_categories: IndexMap::new(),
            },
            profile: empty_profile(),
            path: PathBuf::new(),
            dry_run: true,
        };
        let expected = releases.clone();

        let existing = releases
            .get_or_insert_from_previous("1.0.1".parse().expect("valid semver"))
            .expect("lookup succeeds");

        assert_eq!(
            Some(&existing),
            releases
                .releases
                .get_release(&"1.0.1".parse().expect("valid semver")),
        );
        assert_eq!(
            expected, releases,
            "an existing release must not be duplicated",
        );
    }

    #[test]
    fn get_or_insert_without_previous_creates_empty_release() {
        let mut releases = Releases {
            releases: crate::data::Releases {
                product_name: "OpenTalk".to_owned().into(),
                releases_page_header: None,
                series: BTreeMap::new(),
                components: IndexMap::new(),
                component_categories: IndexMap::new(),
            },
            profile: empty_profile(),
            path: PathBuf::new(),
            dry_run: true,
        };

        let new = releases
            .get_or_insert_from_previous("1.0.0".parse().expect("valid semver"))
            .expect("insert succeeds");

        assert!(
            new.components.is_empty(),
            "the very first release has no components to seed from",
        );
        assert!(
            releases
                .releases
                .get_release(&"1.0.0".parse().expect("valid semver"))
                .is_some()
        );
    }

    #[test]
    fn save_in_dry_run_does_not_write_file() {
        let mut path = std::env::temp_dir();
        path.push("relbo-test.yml");
        let releases = Releases {
            releases: crate::data::Releases {
                product_name: "OpenTalk".to_owned().into(),
                releases_page_header: None,
                series: BTreeMap::new(),
                components: IndexMap::new(),
                component_categories: IndexMap::new(),
            },
            profile: empty_profile(),
            path: path.clone(),
            dry_run: true,
        };

        releases.save().expect("dry-run save is a no-op");

        assert!(!path.exists(), "dry-run must not create the releases file");
    }
}
