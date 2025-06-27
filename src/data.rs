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
    read_profile_file, read_release_file, read_release_file_with_options, write_releases_file,
    ReleaseFileReadOptions,
};
pub use product_name::ProductName;
pub use profiles::{ComponentProfile, Profile};
pub use release::Release;
pub use release_series::ReleaseSeries;
pub use releases::{Releases, StripReleases};
pub use series_number::SeriesNumber;

mod file {
    use std::{
        fs::File,
        io::BufWriter,
        path::{Path, PathBuf},
    };

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

    pub fn read_profile_file(
        release_file: impl AsRef<Path>,
        profile_name: &str,
        profile_path: Option<impl AsRef<Path>>,
    ) -> anyhow::Result<Profile> {
        let profile_path =
            profile_path_from_release_path(release_file, profile_name, profile_path)?;
        let reader: Box<dyn std::io::Read> = if profile_path.as_os_str() == "-" {
            anyhow::bail!("Reading from stdin is not supported in combination with profiles");
        } else {
            Box::new(
                File::open(&profile_path)
                    .with_context(|| format!("Couldn't open profile file {profile_path:?}"))?,
            )
        };

        let profile: Profile = serde_yaml::from_reader::<_, Profile>(reader)?;

        Ok(profile)
    }

    fn profile_path_from_release_path(
        release_file: impl AsRef<Path>,
        profile_name: &str,
        profile_path: Option<impl AsRef<Path>>,
    ) -> anyhow::Result<PathBuf> {
        const PROFILE_DIR: &str = "retoki-profiles";

        let profile_file = if let Some(profile_path) = profile_path {
            profile_path.as_ref().join(format!("{profile_name}.yml"))
        } else {
            let release_file_dir = release_file.as_ref().parent().with_context(|| format!("Could not build profile path for release file `{:?}` since there was no directory containing that file.", release_file.as_ref()))?;
            PathBuf::from(release_file_dir)
                .join(PROFILE_DIR)
                .join(format!("{profile_name}.yml"))
        };

        Ok(profile_file)
    }
}
