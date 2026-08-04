// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::PathBuf;

use clap_complete::engine::CompletionCandidate;

use crate::{
    command::{DEFAULT_RELEASE_FILE, PROFILE_PATH_FLAG, RELEASE_FILE_FLAG},
    data::read_release_file,
};

/// Product release versions, newest first. Empty when the release file is missing or invalid.
pub fn product_version_candidates() -> Vec<CompletionCandidate> {
    let Ok(releases) = read_release_file(completion_context_paths().0) else {
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
    let Ok(releases) = read_release_file(completion_context_paths().0) else {
        return Vec::new();
    };
    releases
        .components
        .keys()
        .map(|identifier| CompletionCandidate::new(identifier.to_string()))
        .collect()
}

/// Extract release.yml and profile-path overwrites from the provided CLI arguments.
///
/// This is used during dynamic autocompletion. We expect retoki to be called with a `--` separator.
/// Everything after the separator is used for the autocompletion and not interpreted as CLI
/// parameter. e.g.: `retoki -- retoki releases 26.1.1 add`
fn completion_context_paths() -> (PathBuf, Option<PathBuf>) {
    let args: Vec<String> = std::env::args_os()
        .filter_map(|arg| arg.into_string().ok())
        .collect();

    // The line being completed is passed after a `--` separator.
    let words = match args.iter().position(|arg| arg == "--") {
        Some(index) => &args[index + 1..],
        None => &[][..],
    };

    let release_file = flag_value(words, RELEASE_FILE_FLAG)
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("RETOKI_RELEASE_FILE").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(DEFAULT_RELEASE_FILE));
    let profile_path = flag_value(words, PROFILE_PATH_FLAG).map(PathBuf::from);

    (release_file, profile_path)
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
