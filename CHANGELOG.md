# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.13.0] - 2026-07-13

[0.13.0]: https://git.opentalk.dev/opentalk/tools/retoki/-/compare/v0.12.0...v0.13.0

### 🚀 New features

- Add `fetch-product-tickets` subcommand ([!433](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/433), [#37](https://git.opentalk.dev/opentalk/tools/retoki/-/issues/37))
- Render product changelog ([!433](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/433), [#38](https://git.opentalk.dev/opentalk/tools/retoki/-/issues/38))
- Move the fetch product ticket command to the release subcommand ([!434](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/434))
- Switch to tracing ([!440](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/440))
- Use tracing-indicatif ([!440](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/440))
- Private components ([!449](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/449))

### 🐛 Bug fixes

- (docs) Add missing `$` in command substitution ([!433](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/433))
- (docs) Fix grammar ([!433](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/433))
- (docs) Fix formatting ([!433](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/433))

### 📦 Dependencies

- (deps) Lock file maintenance ([!432](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/432))
- (deps) Update rust crate tabled to 0.21.0 ([!431](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/431))
- (deps) Ignore unmaintained advisory ([!440](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/440))
- (deps) Update rust crate anyhow to v1.0.103 ([!442](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/442))
- (deps) Update rust crate time to v0.3.52 ([!438](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/438))
- (deps) Update rust crate indicatif to v0.18.6 ([!444](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/444))
- (deps) Update rust crate tera to v2 ([!443](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/443))
- (deps) Update pre-commit hook alessandrojcm/commitlint-pre-commit-hook to v9.26.0 ([!441](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/441))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.19.9 ([!439](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/439))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.20.2 ([!448](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/448))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.97.0 ([!447](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/447))
- (deps) Update pre-commit hook davidanson/markdownlint-cli2 to v0.23.0 ([!435](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/435))
- (deps) Lock file maintenance ([!435](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/435))

## [0.12.0] - 2026-05-29

### 🐛 Bug fixes

- Retoki binary is missing in the container ([!429](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/429))

## [0.11.0] - 2026-05-29

### 🚀 New features

- Document blocking relation between components ([!427](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/427))

### 📦 Dependencies

- (deps) Lock file maintenance ([!424](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/424))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.19.7 ([!422](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/422))
- (deps) Update pre-commit hook alessandrojcm/commitlint-pre-commit-hook to v9.25.0 ([!423](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/423))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.19.8 ([!425](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/425))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.96.0 ([!426](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/426))

## [0.10.0] - 2026-05-22

[0.10.0]: https://git.opentalk.dev/opentalk/tools/retoki/-/compare/v0.9.0...v0.10.0

### 📦 Dependencies

- (deps) Lock file maintenance ([!409](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/409), [!417](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/417), [!418](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/418))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.94.0 ([!408](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/408))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.95.0 ([!415](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/415))

### ⚙ Miscellaneous

- Update project setup ([!421](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/421))

### Ci

- Use changelog template ([!412](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/412))

## [0.8.0] - 2024-10-23
[0.8.0]: https://git.opentalk.dev/opentalk/tools/retoki/-/compare/v0.7.0...v0.8.0

### 🚀 New features

- Add `gitlab_token` CLI argument ([!153](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))
- Don't allow unknown fields in releases data ([!154](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))
- (edit) Remove all changelogs ([!158](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))
- Fetch changelogs in parallel ([!158](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))
- Allow to overwrite selected fields using profiles ([!166](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))
- Remove release code names ([!157](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/), [#21](https://git.opentalk.dev/opentalk/tools/retoki/-/issues/21))

### 🐛 Bug fixes

- Always have a newline above the component changelog ([!156](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/), [#22](https://git.opentalk.dev/opentalk/tools/retoki/-/issues/22))

### ⚡ Performance

- Use `with_context` instead of `context` when building a custom error message ([!149](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))

### 🔨 Refactor

- Introduce function to read releases file ([!149](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))
- Move function `write_releases_file` to module `data::file` ([!149](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))
- Introduce `read_release_file_with_options` helper function ([!149](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))

### 📦 Dependencies

- (deps) Lock file maintenance ([!128](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/)), [!132](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/)), [!138](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/)), [!139](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/)), [!140](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/)), [!141](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/)), [!151](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/)), [!159](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/)), [!162](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/)), [!165](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/)), [!170](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))
- (deps) Update rust crate anyhow to v1.0.91 ([!174](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))
- (deps) Update rust crate clap to v4.5.20 ([!164](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))
- (deps) Update rust crate derive_more to v1 ([!125](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))
- (deps) Update rust crate indexmap to v2.6.0 ([!161](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))
- (deps) Update rust crate owo-colors to v4.1.0 ([!147](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))
- (deps) Update rust crate serde to v1.0.211 ([!172](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))
- (deps) Update rust crate serde_json to v1.0.132 ([!169](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))
- (deps) Update rust crate tabled to 0.16.0 ([!124](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))
- (deps) Update rust crate tempfile to v3.13.0 ([!163](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))

### ⚙ Miscellaneous

- Migrate to new reuse config ([!152](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))

### Ci

- Add commit, toml and yaml lints ([!155](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))
- Introduce changelog bot ([!171](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))

### Test

- Integration test for generating release documentation ([!152](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))
- Integration test for fetching changelog ([!152](https://git.opentalk.dev/opentalk/tools/retoki/-/merge_requests/))
