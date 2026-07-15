// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Render issue bodies for the release workflow.
//!
//! Each function first looks up `.gitlab/issue_templates/<name>.md` in the
//! given project via `VcsService::get_issue_template` and falls back to an
//! embedded default if no such template exists.
//!
//! The embedded defaults mirror the templates introduced in retoki MR !370
//! (`opentalk/tools/retoki`).

use anyhow::Context as _;
use semver::Version;
use serde::Serialize;
use tera::Tera;

use crate::vcs_service::VcsService;

const PRODUCT_RELEASE_DEFAULT: &str = include_str!("bot_templates/product_release.md");
const COMPONENT_RELEASE_DEFAULT: &str = include_str!("bot_templates/component_release.md");

/// A category of components in the product release table.

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CategoryData {
    /// The name of the category e.g., services
    pub name: String,
    /// The components that belong to the category e.g., controller, roomserver
    pub components: Vec<ComponentData>,
}

/// A component row in the product release table.

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ComponentData {
    pub name: String,
    pub version: String,
    pub has_changed: bool,
    pub gitlab_url: Option<String>,
    pub ticket_url: Option<String>,
}

fn render(template: &str, context: &tera::Context) -> anyhow::Result<String> {
    Tera::one_off(template, context, false).context("couldn't render issue template")
}

fn load_template(
    vcs_service: &dyn VcsService,
    project: &str,
    path: &str,
    fallback: &str,
) -> anyhow::Result<String> {
    Ok(vcs_service
        .get_raw_file(project, path)?
        .unwrap_or_else(|| fallback.to_owned()))
}

/// Render the body of the product release issue.
pub(crate) fn product_release_body(
    vcs_service: &dyn VcsService,
    project: &str,
    version: &Version,
    categories: &[CategoryData],
) -> anyhow::Result<String> {
    let template = load_template(
        vcs_service,
        project,
        ".gitlab/issue_templates/product_release.md",
        PRODUCT_RELEASE_DEFAULT,
    )?;

    let mut context = tera::Context::new();
    context.insert("version", version);
    context.insert("categories", categories);

    render(&template, &context)
}

/// Render the body of a component release issue that blocks the product release.
///
/// `component_version` and `previous_version` are expected to already carry the
/// display prefix (e.g. `v1.21.0`) so that the rendered links match the version
/// strings used in the product release table.
#[expect(clippy::too_many_arguments)]
pub(crate) fn component_release_body(
    vcs_service: &dyn VcsService,
    project: &str,
    component_name: &str,
    component_version: &str,
    product_version: &Version,
    category: &str,
    previous_version: Option<&str>,
    gitlab_url: Option<&str>,
) -> anyhow::Result<String> {
    let template = load_template(
        vcs_service,
        project,
        ".gitlab/issue_templates/component_release.md",
        COMPONENT_RELEASE_DEFAULT,
    )?;

    let mut context = tera::Context::new();
    context.insert("component_name", component_name);
    context.insert("component_version", component_version);
    context.insert("product_version", product_version);
    context.insert("category", category);
    context.insert("previous_version", &previous_version);
    context.insert("gitlab_url", &gitlab_url);

    render(&template, &context)
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::vcs_service::MockVcsService;

    fn vcs_with_no_template(name: &'static str) -> MockVcsService {
        let mut vcs = MockVcsService::new();
        let _ = vcs
            .expect_get_raw_file()
            .with(eq("opentalk/tools/relbo"), eq(name))
            .returning(|_, _| Ok(None));
        vcs
    }

    fn vcs_with_template(name: &'static str, body: &'static str) -> MockVcsService {
        let mut vcs = MockVcsService::new();
        let _ = vcs
            .expect_get_raw_file()
            .with(eq("opentalk/tools/relbo"), eq(name))
            .returning(move |_, _| Ok(Some(body.to_owned())));
        vcs
    }

    fn sample_categories() -> Vec<CategoryData> {
        vec![
            CategoryData {
                name: "Frontend".to_owned(),
                components: vec![ComponentData {
                    name: "web-frontend".to_owned(),
                    version: "v2.6.4".to_owned(),
                    has_changed: true,
                    gitlab_url: Some(
                        "https://git.opentalk.dev/opentalk/frontend/web/web-app".to_owned(),
                    ),
                    ticket_url: Some(
                        "https://git.opentalk.dev/opentalk/frontend/web/web-app/-/issues/2801"
                            .to_owned(),
                    ),
                }],
            },
            CategoryData {
                name: "3rd-Party Components".to_owned(),
                components: vec![ComponentData {
                    name: "keycloak".to_owned(),
                    version: "v26.3.2".to_owned(),
                    has_changed: false,
                    gitlab_url: None,
                    ticket_url: None,
                }],
            },
        ]
    }

    #[test]
    fn product_release_uses_embedded_default() {
        let vcs = vcs_with_no_template(".gitlab/issue_templates/product_release.md");
        let version: Version = "25.1.0".parse().unwrap();
        let categories = sample_categories();

        let body =
            product_release_body(&vcs, "opentalk/tools/relbo", &version, &categories).unwrap();

        insta::assert_snapshot!(body);
    }

    #[test]
    fn product_release_uses_repository_template() {
        let vcs = vcs_with_template(
            ".gitlab/issue_templates/product_release.md",
            "Release {{ version }} contains {{ categories | length }} categories.\n",
        );
        let version: Version = "1.2.3".parse().unwrap();

        let body =
            product_release_body(&vcs, "opentalk/tools/relbo", &version, &sample_categories())
                .unwrap();

        assert_eq!(body, "Release 1.2.3 contains 2 categories.\n");
    }
}
