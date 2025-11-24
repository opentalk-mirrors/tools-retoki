// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::fs;

use insta::{assert_debug_snapshot, assert_snapshot};
use retoki::command::{Command, ProfileArgs, generate::GenerateArgs};
use tempfile::tempdir_in;
use time::macros::date;

use crate::for_all_files;

/// With the public profile:
/// 1. create a temporary directory where all output will be written
/// 2. generate release files based on the configuration
/// 3. ensure the output files have the expected content
/// 4. verify that the expected files are written
#[test]
fn test_generate_with_public_profile() {
    // Ensure result directory exists
    let _ = fs::create_dir("tests/test-result");
    let tmp_dir = tempdir_in("tests/test-result").expect("Failed to create temporary directory");
    let command: Command = Command::Generate(GenerateArgs {
        target_dir: tmp_dir.path().to_owned(),
        without_prereleases: false,
        without_readme_gantt_chart: true,
        without_readme_end_of_life: false,
        without_gitlab_release_links: false,
        with_release_metadata_files: true,
        with_md_header: true,
        profile: ProfileArgs {
            profile: "public".to_string(),
            profile_path: None,
        },
        date: Some(date!(2025 - 01 - 01)),
        with_relative_documentation_base_path: None,
    });
    command.execute("tests/generate/releases.yml").unwrap();

    let seen_files = for_all_files(tmp_dir.path(), |path| {
        let input = fs::read_to_string(path).unwrap();
        assert_snapshot!(input);
    });

    assert_debug_snapshot!(seen_files, @r#"
    [
        "24.8.0/README.md",
        "24.8.0/metadata.json",
        "24.8.1/README.md",
        "24.8.1/metadata.json",
        "24.8/README.md",
        "25.0.0/README.md",
        "25.0.0/metadata.json",
        "25.0/README.md",
        "README.md",
        "components/controller.md",
        "components/web-frontend.md",
        "navigation.md",
    ]
    "#);

    tmp_dir.close().expect("Removing tmp dir must work");
}

/// With the private profile:
/// 1. create a temporary directory where all output will be written
/// 2. generate release files based on the configuration
/// 3. ensure the output files have the expected content
/// 4. verify that the expected files are written
#[test]
fn test_generate_with_private_profile() {
    // Ensure result directory exists
    let _ = fs::create_dir("tests/test-result");
    let tmp_dir = tempdir_in("tests/test-result").expect("Failed to create temporary directory");

    let command: Command = Command::Generate(GenerateArgs {
        target_dir: tmp_dir.path().to_owned(),
        without_prereleases: false,
        without_readme_gantt_chart: true,
        without_readme_end_of_life: false,
        without_gitlab_release_links: false,
        with_release_metadata_files: true,
        with_md_header: true,
        profile: ProfileArgs {
            profile: "private".to_string(),
            profile_path: None,
        },
        date: Some(date!(2025 - 01 - 01)),
        with_relative_documentation_base_path: Some("../".to_string()),
    });
    command.execute("tests/generate/releases.yml").unwrap();

    let seen_files = for_all_files(tmp_dir.path(), |path| {
        let input = fs::read_to_string(path).unwrap();
        assert_snapshot!(input);
    });

    assert_debug_snapshot!(seen_files, @r#"
    [
        "24.8.0/README.md",
        "24.8.0/metadata.json",
        "24.8.1/README.md",
        "24.8.1/metadata.json",
        "24.8/README.md",
        "25.0.0/README.md",
        "25.0.0/metadata.json",
        "25.0/README.md",
        "README.md",
        "components/controller.md",
        "components/web-frontend.md",
        "navigation.md",
    ]
    "#);
    tmp_dir.close().expect("Removing tmp dir must work");
}

#[test]
fn test_generate_with_invalid_profile() {
    // Ensure result directory exists
    let _ = fs::create_dir("tests/test-result");
    let tmp_dir = tempdir_in("tests/test-result").expect("Failed to create temporary directory");

    let command: Command = Command::Generate(GenerateArgs {
        target_dir: tmp_dir.path().to_owned(),
        without_prereleases: false,
        without_readme_gantt_chart: true,
        without_readme_end_of_life: false,
        without_gitlab_release_links: false,
        with_release_metadata_files: true,
        with_md_header: true,
        profile: ProfileArgs {
            profile: "invalid".to_string(),
            profile_path: None,
        },
        date: Some(date!(2025 - 01 - 01)),
        with_relative_documentation_base_path: None,
    });
    let err = command.execute("tests/generate/releases.yml").unwrap_err();

    assert_snapshot!(format!("{:?}", err), @r"
    Failed to read profile

    Caused by:
        components.web-frontend: unknown field `gitlab_url_kaputt`, expected `gitlab_url` or `container_base_url` at line 9 column 5
    ");
    tmp_dir.close().expect("Removing tmp dir must work");
}
