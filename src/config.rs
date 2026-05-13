// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use config::{Config as ConfigBuilder, Environment, File, FileFormat, Source};
use serde::{Deserialize, Serialize};
use snafu::{ResultExt as _, Whatever};
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct Config {
    pub gitlab_url: Url,
    pub gitlab_group: String,
    pub gitlab_token: String,
    pub release_label: String,
}
impl Config {
    pub(crate) fn load() -> Result<Self, Whatever> {
        Self::from_sources(
            File::new("relbo", FileFormat::Toml).required(false),
            Environment::with_prefix("RELBO"),
        )
    }

    fn from_sources<F, E>(file: F, env: E) -> Result<Self, Whatever>
    where
        F: Source + Send + Sync + 'static,
        E: Source + Send + Sync + 'static,
    {
        ConfigBuilder::builder()
            .add_source(file)
            .add_source(env)
            .build()
            .whatever_context("couldn't load config")?
            .try_deserialize()
            .whatever_context("couldn't deserialize config")
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
        "#;

        let config =
            Config::from_sources(File::from_str(toml, FileFormat::Toml), empty_env()).unwrap();

        assert_eq!(
            config,
            Config {
                gitlab_url: Url::parse("https://gitlab.example.com/").unwrap(),
                gitlab_group: "my-group".to_owned(),
                gitlab_token: "secret".to_owned(),
                release_label: "release".to_owned(),
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

        let config = Config::from_sources(File::from_str(toml, FileFormat::Toml), env).unwrap();

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

        let config = Config::from_sources(File::from_str("", FileFormat::Toml), env).unwrap();

        assert_eq!(config.gitlab_group, "g");
        assert_eq!(config.gitlab_token, "t");
        assert_eq!(config.release_label, "r");
    }

    fn report(err: Whatever) -> String {
        snafu::Report::from_error(err).to_string()
    }

    #[test]
    fn missing_required_field_errors() {
        let toml = r#"
            gitlab_url = "https://gitlab.example.com/"
            gitlab_group = "my-group"
            gitlab_token = "secret"
            # release_label intentionally missing
        "#;

        let err =
            Config::from_sources(File::from_str(toml, FileFormat::Toml), empty_env()).unwrap_err();

        insta::assert_snapshot!(report(err), @r#"
        couldn't deserialize config

        Caused by this error:
          1: missing configuration field "release_label"
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

        let err =
            Config::from_sources(File::from_str(toml, FileFormat::Toml), empty_env()).unwrap_err();

        insta::assert_snapshot!(report(err), @r#"
        couldn't deserialize config

        Caused by this error:
          1: relative URL without a base: "not a url" for key `gitlab_url`
        "#);
    }
}
