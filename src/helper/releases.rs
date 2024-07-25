// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use semver::Version;

pub(crate) fn is_obsolete_prerelease(all_releases: &BTreeSet<Version>, version: &Version) -> bool {
    let is_final = version.pre.is_empty();
    let final_exists =
        all_releases.contains(&Version::new(version.major, version.minor, version.patch));
    !is_final && final_exists
}
