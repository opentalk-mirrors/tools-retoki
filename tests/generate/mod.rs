// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::fs;

use insta::{assert_debug_snapshot, assert_snapshot};
use retoki::command::{generate::GenerateArgs, Command};
use tempfile::tempdir_in;

use crate::for_all_files;

/// 1. create a temporary directory where all output will be written
/// 2. generate release files based on the configuration
/// 3. ensure the output files have the expected content
/// 4. verify that the expected files are written
#[test]
fn test_generate() {
    // Ensure result directory exists
    let _ = fs::create_dir("tests/test-result");
    let tmp_dir = tempdir_in("tests/test-result").expect("Failed to create temporary directory");

    let command: Command = Command::Generate(GenerateArgs {
        target_dir: tmp_dir.path().to_owned(),
        without_prereleases: false,
        without_readme_gantt_chart: true,
        without_gitlab_release_links: false,
        without_release_series_codenames: true,
        with_release_metadata_files: true,
        with_md_header: true,
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
        "README.md",
        "components/controller.md",
        "components/web-frontend.md",
    ]
    "#);
    tmp_dir.close().expect("Removing tmp dir must work");
}
