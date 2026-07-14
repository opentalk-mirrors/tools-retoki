// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::PathBuf;

use config::{Config as ConfigBuilder, Environment, File, FileFormat, Source};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub(crate) enum ConfigError {
    #[error("couldn't load config")]
    Load(#[source] config::ConfigError),

    #[error("couldn't deserialize config")]
    Deserialize(#[source] config::ConfigError),
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct Config {
    /// Base URL of the GitLab instance to talk to.
    #[serde(default = "default_gitlab_url")]
    pub gitlab_url: Url,

    /// GitLab group under which the managed projects live.
    #[serde(default = "default_gitlab_group")]
    pub gitlab_group: String,

    /// Personal or project access token used to authenticate against GitLab.
    pub gitlab_token: String,

    /// Label used to mark issues that are used to manage the release process.
    #[serde(default = "default_release_label")]
    pub release_label: String,

    /// Full path (group/project) of the GitLab project that tracks product releases.
    #[serde(default = "default_release_repo")]
    pub release_repo: String,

    /// Filesystem path to the `releases.yml` file describing component versions.
    #[serde(default = "default_releases_yml_path")]
    pub releases_yml_path: PathBuf,

    /// Name of the retoki release profile to load alongside `releases.yml`.
    /// The profile file is resolved as `<releases_yml_dir>/retoki-profiles/<release_profile>.yml`.
    #[serde(default = "default_release_profile")]
    pub release_profile: String,
}

fn default_gitlab_url() -> Url {
    Url::parse("https://git.opentalk.dev").expect("hard-coded URL is valid")
}

fn default_gitlab_group() -> String {
    "opentalk".to_string()
}

fn default_release_label() -> String {
    "release-ticket".to_string()
}

fn default_release_repo() -> String {
    "opentalk/product-releases".to_string()
}

fn default_releases_yml_path() -> PathBuf {
    "./releases.yml".parse().expect("hard-coded path is valid")
}

fn default_release_profile() -> String {
    "internal".to_string()
}

impl Config {
    pub(crate) fn load() -> Result<Self, ConfigError> {
        Self::from_sources(
            File::new("relbo", FileFormat::Toml).required(false),
            Environment::with_prefix("RELBO"),
            std::env::var("GITLAB_TOKEN").ok(),
        )
    }

    fn from_sources<F, E>(
        file: F,
        env: E,
        gitlab_token: Option<String>,
    ) -> Result<Self, ConfigError>
    where
        F: Source + Send + Sync + 'static,
        E: Source + Send + Sync + 'static,
    {
        ConfigBuilder::builder()
            .add_source(file)
            .add_source(env)
            .set_override_option("gitlab_token", gitlab_token)
            .map_err(ConfigError::Load)?
            .build()
            .map_err(ConfigError::Load)?
            .try_deserialize()
            .map_err(ConfigError::Deserialize)
    }
}

#[cfg(test)]
mod tests {
    use config::{Environment, File, FileFormat};
    use pretty_assertions::assert_eq;

    use super::*;

    fn empty_env() -> Environment {
        // A prefix that won't match anything to avoid leaking the real process
        // environment into the test.
        Environment::with_prefix("RELBO_TEST_UNUSED_PREFIX_XYZZY")
    }

    #[test]
    fn loads_from_toml() {
        let toml = r#"
            gitlab_url = "https://gitlab.example.com/"
            gitlab_group = "my-group"
            gitlab_token = "secret"
            release_label = "release"
            release_repo = "opentalk/product-releases"
            releases_yml_path = "./releases/releases.yml"
            release_profile = "internal"
        "#;

        let config =
            Config::from_sources(File::from_str(toml, FileFormat::Toml), empty_env(), None)
                .unwrap();

        assert_eq!(
            config,
            Config {
                gitlab_url: Url::parse("https://gitlab.example.com/").unwrap(),
                gitlab_group: "my-group".to_owned(),
                gitlab_token: "secret".to_owned(),
                release_label: "release".to_owned(),
                release_repo: "opentalk/product-releases".to_owned(),
                releases_yml_path: "./releases/releases.yml".parse().expect("Invalid path"),
                release_profile: "internal".to_owned(),
            }
        );
    }

    #[test]
    fn env_overrides_toml() {
        let toml = r#"
            gitlab_url = "https://gitlab.example.com/"
            gitlab_group = "my-group"
            gitlab_token = "secret"
            release_label = "release"
        "#;

        let env = Environment::with_prefix("RELBO_TEST_OVERRIDE").source(Some(
            [
                (
                    "RELBO_TEST_OVERRIDE_GITLAB_TOKEN".to_owned(),
                    "from-env".to_owned(),
                ),
                (
                    "RELBO_TEST_OVERRIDE_RELEASE_LABEL".to_owned(),
                    "hotfix".to_owned(),
                ),
            ]
            .into_iter()
            .collect(),
        ));

        let config =
            Config::from_sources(File::from_str(toml, FileFormat::Toml), env, None).unwrap();

        assert_eq!(config.gitlab_token, "from-env");
        assert_eq!(config.release_label, "hotfix");
        assert_eq!(config.gitlab_group, "my-group");
    }

    #[test]
    fn loads_from_env_only() {
        let env = Environment::with_prefix("RELBO_TEST_ONLY").source(Some(
            [
                (
                    "RELBO_TEST_ONLY_GITLAB_URL".to_owned(),
                    "https://gitlab.example.com/".to_owned(),
                ),
                ("RELBO_TEST_ONLY_GITLAB_GROUP".to_owned(), "g".to_owned()),
                ("RELBO_TEST_ONLY_GITLAB_TOKEN".to_owned(), "t".to_owned()),
                ("RELBO_TEST_ONLY_RELEASE_LABEL".to_owned(), "r".to_owned()),
            ]
            .into_iter()
            .collect(),
        ));

        let config = Config::from_sources(File::from_str("", FileFormat::Toml), env, None).unwrap();

        assert_eq!(config.gitlab_group, "g");
        assert_eq!(config.gitlab_token, "t");
        assert_eq!(config.release_label, "r");
    }

    fn report(err: ConfigError) -> String {
        format!("{:?}", anyhow::Error::from(err))
    }

    #[test]
    fn missing_required_field_errors() {
        let toml = r#"
            # gitlab_token is required if it's not set in the environment
        "#;

        let err = Config::from_sources(File::from_str(toml, FileFormat::Toml), empty_env(), None)
            .unwrap_err();

        insta::assert_snapshot!(report(err), @r#"
        couldn't deserialize config

        Caused by:
            missing configuration field "gitlab_token"
        "#);
    }

    #[test]
    fn invalid_url_errors() {
        let toml = r#"
            gitlab_url = "not a url"
            gitlab_group = "my-group"
            gitlab_token = "secret"
            release_label = "release"
        "#;

        let err = Config::from_sources(File::from_str(toml, FileFormat::Toml), empty_env(), None)
            .unwrap_err();

        insta::assert_snapshot!(report(err), @r#"
        couldn't deserialize config

        Caused by:
            relative URL without a base: "not a url" for key `gitlab_url`
        "#);
    }
}
