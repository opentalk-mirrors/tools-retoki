// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{fs, path::Path};

mod announce;
mod fetch_changelog;
mod generate;

/// Execute the `test_fn` for all files in `base_dir`.
///
/// A sorted list of all seen files is returned.
pub fn for_all_files(base_dir: impl AsRef<Path>, mut test_fn: impl FnMut(&Path)) -> Vec<String> {
    let mut seen_files = Vec::new();
    insta::glob!(
        base_dir.as_ref().to_string_lossy().as_ref(),
        "**/*",
        |path| {
            // Remember which paths we have seen to ensure that the same files get generated.
            if fs::metadata(path).unwrap().is_file() {
                seen_files.push(
                    path.strip_prefix(base_dir.as_ref())
                        .expect("Should have temporary directory as prefix")
                        .display()
                        .to_string(),
                );
                test_fn(path);
            }
        }
    );
    seen_files.sort();
    seen_files
}
