// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use insta::assert_snapshot;
use retoki::command::{ProfileArgs, release::announce::AnnounceArgs};

const RELEASES_FILE: &str = "tests/generate/releases.yml";

fn announce(version: &str) -> String {
    let args = AnnounceArgs {
        profile: ProfileArgs {
            profile: "public".to_string(),
            profile_path: None,
        },
    };
    args.render(RELEASES_FILE, &version.parse().unwrap())
        .unwrap()
}

#[test]
fn announce_markdown_with_release_notes() {
    assert_snapshot!(announce("24.8.0"));
}

#[test]
fn announce_markdown_without_release_notes() {
    assert_snapshot!(announce("25.0.0"));
}
