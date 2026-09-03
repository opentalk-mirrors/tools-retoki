// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::fs;

use insta::{assert_debug_snapshot, assert_snapshot};
use retoki::{
    command::{Command, ProfileArgs, generate::GenerateArgs},
    data::SeriesNumber,
};
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
        series_number: SeriesNumber::from((24, 8)),
        target_dir: tmp_dir.path().to_owned(),
        without_prereleases: false,
        without_readme_gantt_chart: true,
        without_readme_end_of_life: false,
        without_gitlab_release_links: false,
        with_release_metadata_files: true,
        with_md_header: true,
        with_next_release: true,
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
        series_number: SeriesNumber::from((24, 8)),
        target_dir: tmp_dir.path().to_owned(),
        without_prereleases: false,
        without_readme_gantt_chart: true,
        without_readme_end_of_life: false,
        without_gitlab_release_links: false,
        with_release_metadata_files: true,
        with_md_header: true,
        with_next_release: false,
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
        "README.md",
        "components/controller.md",
        "components/web-frontend.md",
        "navigation.md",
    ]
    "#);
    tmp_dir.close().expect("Removing tmp dir must work");
}

/// With a profile that marks the `controller` component as private:
/// 1. create a temporary directory where all output will be written
/// 2. generate release files based on the configuration
/// 3. ensure the output files have the expected content
/// 4. verify that no documentation is written for the private component and that it is excluded
///    from all generated release documentation
#[test]
fn test_generate_with_private_component_profile() {
    // Ensure result directory exists
    let _ = fs::create_dir("tests/test-result");
    let tmp_dir = tempdir_in("tests/test-result").expect("Failed to create temporary directory");

    let command: Command = Command::Generate(GenerateArgs {
        series_number: SeriesNumber::from((24, 8)),
        target_dir: tmp_dir.path().to_owned(),
        without_prereleases: false,
        without_readme_gantt_chart: true,
        without_readme_end_of_life: false,
        without_gitlab_release_links: false,
        with_release_metadata_files: true,
        with_md_header: true,
        with_next_release: false,
        profile: ProfileArgs {
            profile: "private-component".to_string(),
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

    // The private `controller` component must not have a dedicated
    // `components/controller.md` documentation page.
    assert_debug_snapshot!(seen_files, @r#"
    [
        "24.8.0/README.md",
        "24.8.0/metadata.json",
        "24.8.1/README.md",
        "24.8.1/metadata.json",
        "24.8/README.md",
        "README.md",
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
        series_number: SeriesNumber::from((24, 8)),
        target_dir: tmp_dir.path().to_owned(),
        without_prereleases: false,
        without_readme_gantt_chart: true,
        without_readme_end_of_life: false,
        without_gitlab_release_links: false,
        with_release_metadata_files: true,
        with_md_header: true,
        with_next_release: false,
        profile: ProfileArgs {
            profile: "invalid".to_string(),
            profile_path: None,
        },
        date: Some(date!(2025 - 01 - 01)),
        with_relative_documentation_base_path: None,
    });
    let err = command.execute("tests/generate/releases.yml").unwrap_err();

    assert_snapshot!(format!("{:?}", err), @"
    Failed to read profile

    Caused by:
        components.web-frontend: unknown field `gitlab_url_kaputt`, expected one of `gitlab_url`, `container_base_url`, `blocked_by`, `private` at line 9 column 5
    ");
    tmp_dir.close().expect("Removing tmp dir must work");
}
