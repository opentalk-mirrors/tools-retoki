// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::{Path, PathBuf};

use clap_complete::engine::CompletionCandidate;
use semver::Version;

use crate::{
    command::{
        DEFAULT_PROFILE_PATH, DEFAULT_RELEASE_FILE, PROFILE_ENV, PROFILE_FLAG, RELEASE_FILE_FLAG,
    },
    data::{
        ComponentIdentifier, GroupIdentifier, PROFILE_DIR, Releases, is_prerelease,
        read_profile_file, read_release_file,
    },
};

/// Long flags (without leading `--`) that consume the following word as their value.
const VALUE_FLAGS: &[&str] = &[RELEASE_FILE_FLAG, PROFILE_FLAG, "component-version"];
/// Short flags that consume the following word as their value.
const VALUE_SHORT_FLAGS: &[&str] = &["-c"];

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

/// Next patch and minor version for the component (or group) given to `retoki release <version>
/// add`.
///
/// The versions are derived from the component's version in the release preceding the product
/// version, i.e. the latest release of the current release series. Empty when the command line
/// can't be resolved or no previous version of the component is known.
pub fn component_version_candidates() -> Vec<CompletionCandidate> {
    let words = completion_words();
    let Some((product_version, component_or_group)) = release_add_target(&words) else {
        return Vec::new();
    };
    let Ok(releases) = read_release_file(release_file_from_words(&words)) else {
        return Vec::new();
    };

    let component_ids = group_components(&words, component_or_group)
        .unwrap_or_else(|| vec![ComponentIdentifier::from(component_or_group.to_owned())]);

    next_component_versions(&releases, &product_version, &component_ids)
        .into_iter()
        .map(|version| CompletionCandidate::new(version.to_string()))
        .collect()
}

/// The members of `group` if it is a group in the effective profile.
fn group_components(words: &[String], group: &str) -> Option<Vec<ComponentIdentifier>> {
    let profile_path = flag_value(words, PROFILE_FLAG)
        .map(PathBuf::from)
        .or_else(|| std::env::var_os(PROFILE_ENV).map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(DEFAULT_PROFILE_PATH));
    let profile = read_profile_file(&profile_path).ok()?;
    let group = profile
        .groups
        .get(&GroupIdentifier::from(group.to_owned()))?;
    Some(group.components.keys().cloned().collect())
}

/// Product version and component identifier of a `retoki release <version> add <component>`
/// command line.
fn release_add_target(words: &[String]) -> Option<(Version, &str)> {
    let positionals = positional_words(words);
    match positionals.as_slice() {
        ["release", version, "add", component, ..] => {
            Some((Version::parse(version).ok()?, *component))
        }
        _ => None,
    }
}

/// The positional words of the command line, excluding the binary name, flags and flag values.
fn positional_words(words: &[String]) -> Vec<&str> {
    let mut positionals = Vec::new();
    let mut iter = words.iter().skip(1);
    while let Some(word) = iter.next() {
        if let Some(long) = word.strip_prefix("--") {
            if VALUE_FLAGS.contains(&long) {
                iter.next();
            }
        } else if word.starts_with('-') && word.len() > 1 {
            if VALUE_SHORT_FLAGS.contains(&word.as_str()) {
                iter.next();
            }
        } else {
            positionals.push(word.as_str());
        }
    }
    positionals
}

/// Next patch and minor version of the latest version the components had before `product_version`.
///
/// For groups, the highest version among the members is used. If the latest version is a
/// prerelease, its final version is suggested first.
fn next_component_versions(
    releases: &Releases,
    product_version: &Version,
    component_ids: &[ComponentIdentifier],
) -> Vec<Version> {
    let previous_release = releases.previous_release(product_version);
    let Some(base) = component_ids
        .iter()
        .filter_map(|id| {
            previous_release
                .and_then(|release| release.components.get(id))
                // Components not part of the previous release: fall back to their release history.
                .or_else(|| releases.components.get(id)?.releases.keys().max())
                .and_then(|version| version.as_semver())
        })
        .max_by(|a, b| crate::data::prerelease_cmp(a, b))
    else {
        return Vec::new();
    };

    let mut candidates = Vec::new();
    if is_prerelease(base) {
        candidates.push(Version::new(base.major, base.minor, base.patch));
    }
    candidates.push(Version::new(base.major, base.minor, base.patch + 1));
    candidates.push(Version::new(base.major, base.minor + 1, 0));
    candidates
}

/// The words of the command line being completed.
///
/// During dynamic autocompletion retoki is called with a `--` separator. Everything after the
/// separator is the command line being completed, e.g.: `retoki -- retoki release 26.1.1 add`
fn completion_words() -> Vec<String> {
    let args: Vec<String> = std::env::args_os()
        .filter_map(|arg| arg.into_string().ok())
        .collect();

    match args.iter().position(|arg| arg == "--") {
        Some(index) => args[index + 1..].to_vec(),
        None => Vec::new(),
    }
}

/// Resolve the effective release file path from the CLI arguments being completed.
///
/// This is used during dynamic autocompletion. We expect retoki to be called with a `--` separator.
/// Everything after the separator is the command line being completed
/// and not interpreted as a CLI parameter. e.g.: `retoki -- retoki release 26.1.1 add`
fn completion_release_file() -> PathBuf {
    release_file_from_words(&completion_words())
}

fn release_file_from_words(words: &[String]) -> PathBuf {
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const RELEASES: &str = r"
product_name: OpenTalk
series:
  '26.1':
    end_of_life: 2026-12-31
    releases:
      26.1.0:
        date: 2026-01-10
        components:
          controller: 1.4.0
          web-frontend: 2.0.0
      26.1.1:
        date: 2026-02-10
        components:
          controller: 1.4.1
          web-frontend: 2.1.0-beta.1
  '26.2':
    end_of_life: 2027-06-30
    releases:
      26.2.0:
        date: 2026-03-10
        components:
          controller: 1.5.0
          web-frontend: 2.1.0
components:
  controller:
    name: Controller
    category: backend
    releases:
      1.4.0: {}
      1.4.1: {}
      1.5.0: {}
  web-frontend:
    name: Web Frontend
    category: frontend
    releases:
      2.0.0: {}
      2.1.0-beta.1: {}
      2.1.0: {}
  recorder:
    name: Recorder
    category: backend
    releases:
      0.9.0: {}
component_categories:
  backend:
    name: Backend
  frontend:
    name: Frontend
";

    fn releases() -> Releases {
        serde_yaml::from_str(RELEASES).unwrap()
    }

    fn words(line: &str) -> Vec<String> {
        line.split(' ').map(ToOwned::to_owned).collect()
    }

    fn versions(versions: &[&str]) -> Vec<Version> {
        versions
            .iter()
            .map(|v| Version::parse(v).unwrap())
            .collect()
    }

    fn next(product_version: &str, components: &[&str]) -> Vec<Version> {
        let ids: Vec<_> = components
            .iter()
            .map(|id| ComponentIdentifier::from((*id).to_owned()))
            .collect();
        next_component_versions(&releases(), &Version::parse(product_version).unwrap(), &ids)
    }

    #[test]
    fn release_add_target_from_command_line() {
        let words = words(
            "retoki --release-file x.yml release 26.2.1 add controller --profile p.yml --force -c ",
        );
        assert_eq!(
            release_add_target(&words),
            Some((Version::new(26, 2, 1), "controller"))
        );
    }

    #[test]
    fn release_add_target_ignores_other_commands() {
        assert_eq!(
            release_add_target(&words("retoki release 26.2.1 init ")),
            None
        );
        assert_eq!(
            release_add_target(&words("retoki release foo add controller -c ")),
            None
        );
    }

    #[test]
    fn next_versions_in_current_series() {
        assert_eq!(
            next("26.2.1", &["controller"]),
            versions(&["1.5.1", "1.6.0"])
        );
    }

    #[test]
    fn next_versions_for_hotfix_of_older_series() {
        assert_eq!(
            next("26.1.2", &["controller"]),
            versions(&["1.4.2", "1.5.0"])
        );
    }

    #[test]
    fn next_versions_after_prerelease_include_final_version() {
        assert_eq!(
            next("26.1.2", &["web-frontend"]),
            versions(&["2.1.0", "2.1.1", "2.2.0"])
        );
    }

    #[test]
    fn next_versions_for_group_use_highest_member() {
        assert_eq!(
            next("26.2.1", &["controller", "web-frontend"]),
            versions(&["2.1.1", "2.2.0"])
        );
    }

    #[test]
    fn next_versions_fall_back_to_component_history() {
        assert_eq!(
            next("26.2.1", &["recorder"]),
            versions(&["0.9.1", "0.10.0"])
        );
    }

    #[test]
    fn next_versions_for_unknown_component() {
        assert_eq!(next("26.2.1", &["unknown"]), Vec::<Version>::new());
    }
}
