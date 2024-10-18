// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::fs;

use insta::{assert_debug_snapshot, assert_snapshot};
use retoki::command::{
    release::{FetchChangelogsArgs, ReleaseArgs, ReleaseCommand},
    Command, ProfileArgs,
};
use tempfile::tempdir_in;

use crate::for_all_files;

/// 1. create a temporary directory
/// 2. move a copy of releases.yml to the directory (ensure to not change the original)
/// 3. execute fetch changelog
///     * this actually doesn't fetch changelogs since no gitlab url is given
///     * only the version information is added to the component
/// 4. ensure the files have the expected output
#[test]
fn test_fetch_changelog_with_public_profile() {
    // SETUP: Ensure result directory exists, create a temporary directory, copy
    //        release.yml since it will be changed (changelog added by retoki)
    let _ = fs::create_dir("tests/test-result");
    let tmp_dir = tempdir_in("tests/test-result").expect("Failed to create temporary directory");
    let tmp_release_yml = tmp_dir.path().join("releases.yml");
    fs::copy("tests/fetch_changelog/releases.yml", &tmp_release_yml)
        .expect("Could not create copy of release.yml");

    // Fetch the changelog. Only the changelog should be added in the release.yml.
    let command: Command = Command::Release(ReleaseArgs {
        version: "24.8.0".parse().expect("Must be a valid version"),
        command: ReleaseCommand::FetchChangelogs(FetchChangelogsArgs {
            gitlab_token: "Dummy".to_owned(),
            profile: ProfileArgs {
                profile: "public".to_owned(),
                profile_path: Some(
                    "tests/fetch_changelog/retoki-profiles"
                        .parse()
                        .expect("Must be valid path"),
                ),
            },
        }),
    });
    command.execute(&tmp_release_yml).unwrap();

    let seen_files = for_all_files(tmp_dir.path(), |path| {
        let input = fs::read_to_string(path).unwrap();
        assert_snapshot!(input);
    });

    assert_debug_snapshot!(seen_files, @r#"
    [
        "releases.yml",
    ]
    "#);
    tmp_dir.close().expect("Removing tmp dir must work");
}
