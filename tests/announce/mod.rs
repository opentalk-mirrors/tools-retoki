// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::PathBuf;

use insta::assert_snapshot;
use retoki::command::{ProfileArgs, release::announce::AnnounceArgs};

const RELEASES_FILE: &str = "tests/generate/releases.yml";

fn announce(version: &str) -> String {
    let args = AnnounceArgs {
        profile_args: ProfileArgs {
            profile: PathBuf::from("tests/generate/retoki-profiles/public.yml"),
        },
    };
    args.render(RELEASES_FILE, &version.parse().unwrap())
        .unwrap()
}

#[test]
fn announce_plaintext_with_release_notes() {
    assert_snapshot!(announce("24.8.0"));
}

#[test]
fn announce_plaintext_without_release_notes() {
    assert_snapshot!(announce("25.0.0"));
}
