use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct Config {}

mod figment_impls {
    use figment::{
        providers::{Env, Format as _, Toml},
        Error, Figment, Provider,
    };

    use super::*;

    impl Config {
        fn from<T: Provider>(provider: T) -> Result<Self, Error> {
            Figment::from(provider).extract()
        }

        pub(crate) fn figment() -> Figment {
            Figment::new()
                .merge(Toml::file("relbo.toml"))
                .merge(Env::prefixed("RELBO_"))
        }

        pub(crate) fn load() -> Self {
            Self::from(Self::figment()).expect("valid config")
        }
    }
}
