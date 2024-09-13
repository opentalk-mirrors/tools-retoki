// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

mod component;
mod component_category;
mod component_category_identifier;
mod component_category_name;
mod component_identifier;
mod component_name;
mod component_release;
mod component_version;
mod product_name;
mod release;
mod release_series;
mod releases;
mod series_codename;
mod series_number;

pub use component::Component;
pub use component_category::ComponentCategory;
pub use component_category_identifier::ComponentCategoryIdentifier;
pub use component_category_name::ComponentCategoryName;
pub use component_identifier::ComponentIdentifier;
pub use component_name::ComponentName;
pub use component_release::ComponentRelease;
pub use component_version::ComponentVersion;
pub use file::{read_release_file, ReleaseFileReadOptions, ReleaseSeriesCodenames};
pub use product_name::ProductName;
pub use release::Release;
pub use release_series::ReleaseSeries;
pub use releases::{Releases, StripReleases};
pub use series_codename::SeriesCodename;
pub use series_number::SeriesNumber;

mod file {
    use std::{fs::File, path::Path};

    use anyhow::Context as _;

    use crate::data::{self, Releases, StripReleases};

    /// Configures whether the release series codenames are striped or kept.
    #[derive(Debug, Default, PartialEq, Eq)]
    pub enum ReleaseSeriesCodenames {
        #[default]
        Keep,
        Strip,
    }

    impl ReleaseSeriesCodenames {
        /// Returns `true` if the release series codenames is [`Strip`].
        ///
        /// [`Strip`]: ReleaseSeriesCodenames::Strip
        #[must_use]
        pub fn is_strip(&self) -> bool {
            matches!(self, Self::Strip)
        }
    }

    #[derive(Debug, Default)]
    pub struct ReleaseFileReadOptions {
        pub strip_prereleases: Option<StripReleases>,
        pub release_series_codenames: ReleaseSeriesCodenames,
    }

    pub fn read_release_file(
        release_file: impl AsRef<Path>,
        options: ReleaseFileReadOptions,
    ) -> anyhow::Result<Releases> {
        let reader: Box<dyn std::io::Read> = if release_file.as_ref().as_os_str() == "-" {
            Box::new(std::io::stdin().lock())
        } else {
            Box::new(File::open(&release_file).with_context(|| {
                format!("Couldn't open release file {:?}", release_file.as_ref())
            })?)
        };

        let mut raw_data: data::Releases = serde_yaml::from_reader::<_, data::Releases>(reader)?;

        if let Some(strip_prereleases) = options.strip_prereleases {
            raw_data = raw_data.with_releases_stripped(strip_prereleases);
        }

        if options.release_series_codenames.is_strip() {
            raw_data.strip_release_series_codenames();
        }

        Ok(raw_data)
    }
}
