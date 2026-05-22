// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Classifying and comparing semver [`Version`]s with respect to their pre-release identifier.
//!
//! A `pre` consisting only of ASCII digits (e.g. `0.16.1-1`) is treated as a
//! *post-release* of the base version rather than a real pre-release. Any
//! other non-empty `pre` (e.g. `0.16.1-rc.1`) is a real pre-release.

use semver::{Prerelease, Version};

/// Classification of a [`Prerelease`] identifier.
///
/// The ordering is significant: `Real < Empty < Numeric`, so a real
/// pre-release sorts below the final release and a numeric "post-release"
/// suffix sorts above it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PreKind {
    /// Real pre-release identifier (contains at least one non-digit), e.g. `rc.1`.
    Real,
    /// Empty pre-release identifier (final release).
    Empty,
    /// Purely numeric pre-release identifier, treated as a post-release.
    Numeric,
}

pub fn pre_kind(pre: &Prerelease) -> PreKind {
    if pre.is_empty() {
        PreKind::Empty
    } else if pre.as_str().bytes().all(|b| b.is_ascii_digit()) {
        PreKind::Numeric
    } else {
        PreKind::Real
    }
}

/// Returns `true` iff `version` is a real pre-release.
pub fn is_prerelease(version: &Version) -> bool {
    pre_kind(&version.pre) == PreKind::Real
}
