// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use indexmap::IndexMap;
use semver::Version;

pub(crate) fn is_obsolete_prerelease<T>(
    all_releases: &IndexMap<Version, T>,
    version: &Version,
) -> bool {
    let is_final = version.pre.is_empty();
    let final_exists =
        all_releases.contains_key(&Version::new(version.major, version.minor, version.patch));
    !is_final && final_exists
}
