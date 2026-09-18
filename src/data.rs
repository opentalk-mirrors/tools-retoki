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
mod group_identifier;
mod prerelease;
mod product_name;
mod product_ticket;
mod profiles;
mod release;
mod release_series;
mod releases;
mod series_number;

pub use component::Component;
pub use component_category::ComponentCategory;
pub use component_category_identifier::ComponentCategoryIdentifier;
pub use component_category_name::ComponentCategoryName;
pub use component_identifier::ComponentIdentifier;
pub use component_name::ComponentName;
pub use component_release::ComponentRelease;
pub use component_version::ComponentVersion;
pub use file::{
    ReleaseFileReadOptions, read_profile_file, read_release_file, read_release_file_with_options,
    write_releases_file,
};
pub use group_identifier::GroupIdentifier;
pub use prerelease::is_prerelease;
pub use product_name::ProductName;
pub use product_ticket::ProductTicket;
pub use profiles::{ComponentGroup, Profile};
pub use release::Release;
pub use release_series::ReleaseSeries;
pub use releases::{Releases, StripReleases};
pub use series_number::SeriesNumber;

mod file {
    use std::{fs::File, io::BufWriter, path::Path};

    use anyhow::Context as _;

    use super::profiles::Profile;
    use crate::data::{Releases, StripReleases};

    #[derive(Debug, Default)]
    pub struct ReleaseFileReadOptions {
        pub strip_prereleases: Option<StripReleases>,
    }

    pub fn read_release_file(release_file: impl AsRef<Path>) -> anyhow::Result<Releases> {
        read_release_file_with_options(release_file, Default::default())
    }

    pub fn read_release_file_with_options(
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

        let mut data: Releases = serde_yaml::from_reader(reader)?;

        if let Some(strip_prereleases) = options.strip_prereleases {
            data = data.with_releases_stripped(strip_prereleases);
        }

        Ok(data)
    }

    pub fn write_releases_file(
        release_file: impl AsRef<Path>,
        data: Releases,
    ) -> anyhow::Result<()> {
        let file = File::create(&release_file).with_context(|| {
            format!("Couldn't write to release file {:?}", release_file.as_ref())
        })?;
        let writer = BufWriter::new(file);

        serde_yaml::to_writer(writer, &data)
            .with_context(|| format!("Couldn't write releases file {:?}", release_file.as_ref()))?;

        Ok(())
    }

    pub fn read_profile_file(profile_path: &Path) -> anyhow::Result<Profile> {
        let reader: Box<dyn std::io::Read> = if profile_path.as_os_str() == "-" {
            anyhow::bail!("Reading from stdin is not supported in combination with profiles");
        } else {
            Box::new(
                File::open(profile_path)
                    .with_context(|| format!("Couldn't open profile file {profile_path:?}"))?,
            )
        };

        let profile: Profile = serde_yaml::from_reader::<_, Profile>(reader)?;

        Ok(profile)
    }
}
