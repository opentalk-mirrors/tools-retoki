# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
