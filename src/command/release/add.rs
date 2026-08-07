// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
use std::io::stdout;

use anyhow::{Context as _, Ok};
use clap::Args;
use semver::Version;

use crate::{
    bot_config::Config,
    bot_templates::{self, product_release_body},
    data::ComponentVersion,
    gitlab_service::GitlabService,
    output::Output,
    release_workflow::{Releases, ReleasesBuilder, ResolvedComponent, build_release_title},
    vcs_service::{Issue, IssueLinkType, LinkedIssue, VcsService, VcsServiceExt},
};

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct AddArgs {
    /// Identifier of the component to add, i.e. its key in `releases.yml`.
    component: String,

    /// Planned version of the component for this release.
    #[arg(long, short = 'c')]
    component_version: Version,

    /// Only log the actions that would be performed without writing anything.
    #[arg(long, env = "RETOKI_DRY_RUN")]
    dry_run: bool,
}

impl AddArgs {
    pub fn execute(&self, version: &Version) -> anyhow::Result<()> {
        let config = Config::load()?;
        let vcs = GitlabService::connect(
            config.gitlab_url.clone(),
            config.gitlab_token.clone(),
            config.gitlab_group.clone(),
        )?
        .dry_run_if(self.dry_run);

        self.run_inner(version, &config, &vcs, &mut stdout().lock())
    }

    fn run_inner(
        &self,
        product_version: &Version,
        config: &Config,
        vcs: &dyn VcsService,
        out: &mut dyn Output,
    ) -> anyhow::Result<()> {
        let mut releases = ReleasesBuilder::new()
            .dry_run(self.dry_run)
            .load(config.releases_yml_path.clone(), &config.release_profile)?;

        let resolved = releases.resolve_component(&self.component)?;

        let title = format!("Release {product_version}");
        let product_issue = vcs
            .get_open_issue_with_title(&config.release_repo, &title)?
            .with_context(|| {
                format!(
                    "no open product release issue {title:?} found in {repo}; \
                     run `retoki release {product_version} init` first",
                    repo = config.release_repo,
                )
            })?;

        let component_version = ComponentVersion::Semver(self.component_version.clone());

        // Start from the product issue's current blockers. A freshly created
        // and linked component issue is appended below so that the re-rendered
        // product release table can resolve its ticket link.
        let mut linked_issues = product_issue.linked_issues.clone();

        if let Some(gitlab_url) = resolved.gitlab_url.as_deref() {
            self.put_component_issue(
                product_version,
                config,
                vcs,
                out,
                &releases,
                &resolved,
                &product_issue,
                &component_version,
                &mut linked_issues,
                gitlab_url,
            )?;
        } else {
            out.println(&format_args!(
                "Component {component} has no gitlab project configured; \
                 recording the version without a release issue",
                component = self.component,
            ));
        }

        let release = releases
            .edit(|r| r.set_component_version(product_version, resolved.id, component_version))?
            .save()?;

        let categories: Vec<_> = releases
            .category_data(product_version, &release, &linked_issues, vcs)?
            .collect();
        let body = product_release_body(vcs, &config.release_repo, product_version, &categories)?;
        vcs.update_issue_description(&config.release_repo, product_issue.iid, &body)?;

        out.println(&format_args!(
            "Updated product release issue {path}#{iid}",
            path = product_issue.project.path_with_namespace,
            iid = product_issue.iid,
        ));

        Ok(())
    }

    #[expect(clippy::too_many_arguments)]
    fn put_component_issue(
        &self,
        product_version: &Version,
        config: &Config,
        vcs: &dyn VcsService,
        out: &mut dyn Output,
        releases: &Releases,
        resolved: &ResolvedComponent,
        product_issue: &Issue,
        component_version: &ComponentVersion,
        linked_issues: &mut Vec<LinkedIssue>,
        gitlab_url: &str,
    ) -> Result<(), anyhow::Error> {
        let component_project = component_project(vcs, gitlab_url, &self.component)?;
        let component_title = build_release_title(&resolved.name, component_version);
        let component_issue = match find_existing_component_issue(
            vcs,
            product_issue,
            &component_project,
            &config.release_label,
            &component_title,
            component_version,
        )? {
            Some(existing) => {
                out.println(&format_args!(
                    "Reusing existing component release issue {reference}",
                    reference = existing.reference_with_url(),
                ));
                existing
            }
            None => {
                let previous_version = releases
                    .previous_component_version(product_version, &resolved.id)
                    .map(|previous| ComponentVersion::Semver(previous).prefixed());
                let body = bot_templates::component_release_body(
                    vcs,
                    &component_project,
                    &resolved.name.to_string(),
                    component_version,
                    product_version,
                    &resolved.category_name,
                    previous_version.as_deref(),
                    Some(gitlab_url),
                )?;
                let created = vcs.create_issue(
                    &component_project,
                    &component_title,
                    &body,
                    &[&config.release_label],
                )?;
                out.println(&format_args!(
                    "Created component release issue {reference}",
                    reference = created.reference_with_url(),
                ));
                created
            }
        };
        let already_linked = product_issue.linked_issues.iter().any(|linked| {
            linked.link_type == IssueLinkType::IsBlockedBy
                && linked.issue.project.path_with_namespace == component_project
                && linked.issue.iid == component_issue.iid
        });

        if already_linked {
            out.println(&format_args!(
                "Product release issue is already blocked by {path}#{iid}",
                path = component_issue.project.path_with_namespace,
                iid = component_issue.iid,
            ));
        } else {
            vcs.create_issue_link(
                &config.release_repo,
                product_issue.iid,
                &component_project,
                component_issue.iid,
                IssueLinkType::IsBlockedBy,
            )?;
            out.println(&format_args!(
                "Linked product release issue to be blocked by {path}#{iid}",
                path = component_issue.project.path_with_namespace,
                iid = component_issue.iid,
            ));
            linked_issues.push(LinkedIssue {
                link_type: IssueLinkType::IsBlockedBy,
                issue: component_issue,
            });
        }
        Ok(())
    }
}

/// Derive the project path (`group/subgroup/name`) of a component from the
/// `gitlab_url` declared in its release profile.
fn component_project(
    vcs: &dyn VcsService,
    gitlab_url: &str,
    component: &str,
) -> anyhow::Result<String> {
    let url = gitlab_url
        .parse()
        .with_context(|| format!("invalid gitlab_url {gitlab_url:?} for component {component}"))?;
    vcs.project_path_from_url(&url)
        .with_context(|| format!("couldn't derive project path from gitlab_url {gitlab_url:?}"))
}

/// Look for an existing component release issue so that `add` stays idempotent.
///
/// First scans the product release's blockers that live in the component's
/// project, then falls back to searching the component project for issues
/// carrying the release label. Within each set an exact title match wins;
/// otherwise a title containing the version string is accepted with a warning.
fn find_existing_component_issue(
    vcs: &dyn VcsService,
    product_issue: &Issue,
    component_project: &str,
    release_label: &str,
    expected_title: &str,
    version_marker: &ComponentVersion,
) -> anyhow::Result<Option<Issue>> {
    let version_marker = version_marker.to_string();
    let linked = product_issue
        .linked_issues
        .iter()
        .filter(|linked| linked.link_type == IssueLinkType::IsBlockedBy)
        .map(|linked| &linked.issue)
        .filter(|issue| issue.project.path_with_namespace == component_project);

    if let Some(found) = match_component_issue(linked, expected_title, &version_marker) {
        return Ok(Some(found.clone()));
    }

    let candidates = vcs.find_issues_with_label(component_project, release_label)?;
    Ok(match_component_issue(candidates.iter(), expected_title, &version_marker).cloned())
}

/// Match a component release issue among `candidates`, preferring an exact
/// title match and falling back to any title containing `version_marker`.
fn match_component_issue<'a, I>(
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
        .find(|i| i.title.contains(version_marker))?;
    tracing::warn!(
        existing_title = %fuzzy.title,
        expected_title,
        "matched existing component release issue by version substring in title; \
         consider renaming the ticket to the expected title",
    );
    Some(fuzzy)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::{Arc, Mutex},
    };

    use pretty_assertions::assert_eq;
    use tempfile::tempdir;
    use url::Url;

    use super::*;
    use crate::{
        bot_config::Config,
        vcs_service::{IssueState, MockVcsService, Project},
    };

    const SAMPLE: &str = r#"---
product_name: OpenTalk
series:
  '25.1':
    end_of_life: 2026-06-30
    releases:
      25.1.0:
        date: 2025-08-01
        components:
          web-frontend: 1.20.0
          controller: 0.18.0
  '25.0':
    end_of_life: 2026-01-01
    releases:
      25.0.0:
        date: 2025-07-01
        components:
          web-frontend: 1.20.0
          controller: 0.18.0
component_categories:
  frontend:
    name: Frontend
  services:
    name: Services
components:
  web-frontend:
    name: Web-Frontend
    category: frontend
  controller:
    name: Controller
    category: services
"#;

    const PROFILE: &str = r#"---
profile_name: internal
components:
  web-frontend:
    gitlab_url: https://gitlab.example.com/opentalk/web-frontend
  controller:
    gitlab_url: https://gitlab.example.com/opentalk/controller
"#;

    fn write_sample(dir: &Path) -> PathBuf {
        let path = dir.join("releases.yml");
        fs::write(&path, SAMPLE).unwrap();
        let profile_dir = dir.join("retoki-profiles");
        fs::create_dir_all(&profile_dir).unwrap();
        fs::write(profile_dir.join("internal.yml"), PROFILE).unwrap();
        path
    }

    fn config_for(path: PathBuf) -> Config {
        Config {
            gitlab_url: Url::parse("https://gitlab.example.com").unwrap(),
            gitlab_group: "opentalk".to_owned(),
            gitlab_token: "token".to_owned(),
            release_label: "release".to_owned(),
            release_repo: "opentalk/product-releases".to_owned(),
            releases_yml_path: path,
            release_profile: "internal".to_owned(),
        }
    }

    fn add_args(dry_run: bool) -> AddArgs {
        AddArgs {
            component: "web-frontend".to_owned(),
            component_version: "1.21.0".parse().unwrap(),
            dry_run,
        }
    }

    fn product_issue() -> Issue {
        Issue {
            id: 2000,
            iid: 7,
            title: "Release 25.1.0".to_owned(),
            project: Project {
                id: 1,
                path_with_namespace: "opentalk/product-releases".to_owned(),
            },
            short_reference: "opentalk/product-releases#7".to_owned(),
            description: Some("old body".to_owned()),
            state: IssueState::Opened,
            linked_issues: Vec::new(),
            web_url: "https://gitlab.example.com/opentalk/product-releases/-/issues/7"
                .parse()
                .unwrap(),
        }
    }

    fn component_issue() -> Issue {
        Issue {
            id: 3000,
            iid: 11,
            title: "Release v1.21.0 of Web-Frontend".to_owned(),
            project: Project {
                id: 2,
                path_with_namespace: "opentalk/web-frontend".to_owned(),
            },
            short_reference: "opentalk/web-frontend#11".to_owned(),
            description: None,
            state: IssueState::Opened,
            linked_issues: Vec::new(),
            web_url: "https://gitlab.example.com/opentalk/web-frontend/-/issues/11"
                .parse()
                .unwrap(),
        }
    }

    /// Common read-side stubs shared by all scenarios where the product
    /// release issue exists.
    fn stub_reads(vcs: &mut MockVcsService) {
        let _ = vcs
            .expect_get_issue()
            .returning(|_, _| Ok(Some(product_issue())));
        let _ = vcs
            .expect_get_open_issue_with_title()
            .returning(|_, _| Ok(Some(product_issue())));
        let _ = vcs
            .expect_project_path_from_url()
            .returning(|url: &Url| Ok(url.path().trim_matches('/').to_owned()));
        let _ = vcs
            .expect_find_issues_with_label()
            .returning(|_, _| Ok(Vec::new()));
        let _ = vcs.expect_get_raw_file().returning(|_, _| Ok(None));
    }

    #[test]
    fn add_creates_component_issue_links_and_updates_product_body() {
        let dir = tempdir().unwrap();
        let path = write_sample(dir.path());
        let config = config_for(path.clone());

        let mut vcs = MockVcsService::new();
        stub_reads(&mut vcs);
        let _ = vcs
            .expect_create_issue()
            .withf(|project, title, _body, labels| {
                project == "opentalk/web-frontend"
                    && title == "Release v1.21.0 of Web-Frontend"
                    && labels == ["release"]
            })
            .returning(|_, _, _, _| Ok(component_issue()));
        let _ = vcs
            .expect_create_issue_link()
            .withf(
                |source_project, source_iid, target_project, target_iid, link_type| {
                    source_project == "opentalk/product-releases"
                        && *source_iid == 7
                        && target_project == "opentalk/web-frontend"
                        && *target_iid == 11
                        && *link_type == IssueLinkType::IsBlockedBy
                },
            )
            .returning(|_, _, _, _, _| Ok(()));
        let captured: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let captured_clone = Arc::clone(&captured);
        let _ = vcs
            .expect_update_issue_description()
            .withf(|project, iid, _| project == "opentalk/product-releases" && *iid == 7)
            .returning(move |_, _, body| {
                *captured_clone.lock().unwrap() = Some(body.to_owned());
                Ok(())
            });

        let mut out = Vec::new();
        add_args(false)
            .run_inner(&"25.1.0".parse().unwrap(), &config, &vcs, &mut out)
            .unwrap();

        // releases.yml was updated with the new component version.
        let written = fs::read_to_string(&path).unwrap();
        assert!(
            written.contains("web-frontend: 1.21.0"),
            "releases.yml should record the new version, got:\n{written}",
        );

        // The product release table links to the freshly created component issue.
        let body = captured.lock().unwrap().clone().expect("body was updated");
        assert!(
            body.contains("https://gitlab.example.com/opentalk/web-frontend/-/issues/11"),
            "product body should link the component issue, got:\n{body}",
        );
    }

    #[test]
    fn add_is_idempotent_when_component_issue_already_exists() {
        let dir = tempdir().unwrap();
        let path = write_sample(dir.path());
        let config = config_for(path);

        let mut vcs = MockVcsService::new();
        let _ = vcs
            .expect_get_open_issue_with_title()
            .returning(|_, _| Ok(Some(product_issue())));
        let _ = vcs
            .expect_project_path_from_url()
            .returning(|url: &Url| Ok(url.path().trim_matches('/').to_owned()));
        // An issue with the expected title already carries the release label.
        let _ = vcs
            .expect_find_issues_with_label()
            .returning(|_, _| Ok(vec![component_issue()]));
        let _ = vcs.expect_get_raw_file().returning(|_, _| Ok(None));
        // No new issue is created, but the existing one is linked as a blocker.
        let _ = vcs
            .expect_create_issue_link()
            .returning(|_, _, _, _, _| Ok(()));
        let _ = vcs
            .expect_update_issue_description()
            .returning(|_, _, _| Ok(()));

        let mut out = Vec::new();
        add_args(false)
            .run_inner(&"25.1.0".parse().unwrap(), &config, &vcs, &mut out)
            .unwrap();

        let printed = String::from_utf8(out).unwrap();
        assert!(
            printed.contains("Reusing existing component release issue"),
            "expected a reuse notice, got:\n{printed}",
        );
    }

    #[test]
    fn add_dry_run_does_not_write_releases_or_issues() {
        let dir = tempdir().unwrap();
        let path = write_sample(dir.path());
        let original = fs::read_to_string(&path).unwrap();
        let config = config_for(path.clone());

        let mut vcs = MockVcsService::new();
        stub_reads(&mut vcs);
        // Write operations must never reach the mock in dry-run mode; they are
        // intercepted by the dry-run decorator instead.

        let dry_run_vcs = vcs.dry_run_if(true);

        let mut out = Vec::new();
        add_args(true)
            .run_inner(&"25.1.0".parse().unwrap(), &config, &dry_run_vcs, &mut out)
            .unwrap();

        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            original,
            "releases.yml must be untouched in dry-run mode",
        );
    }

    #[test]
    fn add_fails_when_product_release_issue_is_missing() {
        let dir = tempdir().unwrap();
        let path = write_sample(dir.path());
        let config = config_for(path);

        let mut vcs = MockVcsService::new();
        let _ = vcs
            .expect_get_open_issue_with_title()
            .returning(|_, _| Ok(None));

        let mut out = Vec::new();
        let err = add_args(false)
            .run_inner(&"25.1.0".parse().unwrap(), &config, &vcs, &mut out)
            .unwrap_err();

        assert!(
            format!("{err:#}").contains("retoki release 25.1.0 init"),
            "error should point at init, got: {err:#}",
        );
    }
}
