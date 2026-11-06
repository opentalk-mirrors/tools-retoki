// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::{Path, PathBuf};

use clap_complete::engine::CompletionCandidate;

use crate::{
    command::{DEFAULT_RELEASE_FILE, RELEASE_FILE_FLAG},
    data::{PROFILE_DIR, read_release_file},
};

/// Product release versions, newest first. Empty when the release file is missing or invalid.
pub fn product_version_candidates() -> Vec<CompletionCandidate> {
    let Ok(releases) = read_release_file(completion_release_file()) else {
        return Vec::new();
    };
    releases
        .all_product_versions()
        .iter()
        .rev()
        .map(|version| CompletionCandidate::new(version.to_string()))
        .collect()
}

/// Component identifiers. Empty when the release file is missing or invalid.
pub fn component_candidates() -> Vec<CompletionCandidate> {
    let Ok(releases) = read_release_file(completion_release_file()) else {
        return Vec::new();
    };
    releases
        .components
        .keys()
        .map(|identifier| CompletionCandidate::new(identifier.to_string()))
        .collect()
}

/// Profile names from existing files in the effective profile directory.
///
/// We don't autocomplete the full name, since retoki only uses the file name without suffix as
/// profile name.
pub fn profile_candidates() -> Vec<CompletionCandidate> {
    let release_file = completion_release_file();
    let profile_dir = release_file
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(PROFILE_DIR);

    let Ok(entries) = std::fs::read_dir(profile_dir) else {
        return Vec::new();
    };

    // Filter out files which don't qualify as profile configurations
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let is_yaml = path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| {
                    ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml")
                });
            if !path.is_file() || !is_yaml {
                return None;
            }

            path.file_stem()
                .and_then(|stem| stem.to_str())
                .map(ToOwned::to_owned)
        })
        .collect();

    names.sort();
    names.into_iter().map(CompletionCandidate::new).collect()
}

/// Resolve the effective release file path from the CLI arguments being completed.
///
/// This is used during dynamic autocompletion. We expect retoki to be called with a `--` separator.
/// Everything after the separator is the command line being completed
/// and not interpreted as a CLI parameter. e.g.: `retoki -- retoki release 26.1.1 add`
fn completion_release_file() -> PathBuf {
    let args: Vec<String> = std::env::args_os()
        .filter_map(|arg| arg.into_string().ok())
        .collect();

    // The line being completed is passed after a `--` separator.
    let words = match args.iter().position(|arg| arg == "--") {
        Some(index) => &args[index + 1..],
        None => &[][..],
    };

    flag_value(words, RELEASE_FILE_FLAG)
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("RETOKI_RELEASE_FILE").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(DEFAULT_RELEASE_FILE))
}

/// Last value passed for `flag`, in either `--flag value` or `--flag=value` form.
fn flag_value(words: &[String], flag: &str) -> Option<String> {
    let long = format!("--{flag}");
    let prefix = format!("--{flag}=");
    words.iter().enumerate().rev().find_map(|(i, word)| {
        // `--flag=value`
        if let Some(value) = word.strip_prefix(&prefix) {
            Some(value.to_owned())

        // [..., '--flag', 'value', ...]
        } else if *word == long {
            words.get(i + 1).cloned()
        } else {
            None
        }
    })
}
