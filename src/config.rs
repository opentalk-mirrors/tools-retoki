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
