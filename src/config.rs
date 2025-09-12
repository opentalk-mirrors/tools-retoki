// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

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

mod figment_impls {
    use figment::{
        providers::{Env, Format as _, Toml},
        Error, Figment, Provider,
    };

    use super::*;

    impl Config {
        // The error variant is in a box since the error type is at least 208 bytes.
        fn from<T: Provider>(provider: T) -> Result<Self, Box<Error>> {
            Figment::from(provider).extract().map_err(Box::new)
        }

        pub(crate) fn figment() -> Figment {
            Figment::new()
                .merge(Toml::file("relbo.toml"))
                .merge(Env::prefixed("RELBO_"))
        }

        pub(crate) fn load() -> Result<Self, Whatever> {
            Self::from(Self::figment()).whatever_context("couldn't load config")
        }
    }
}
