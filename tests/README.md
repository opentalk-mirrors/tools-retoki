# Integration test utilizing [Insta](https://insta.rs/)

The integration tests use snapshot testing to ensure the output doesn't change
unexpectedly.

## Setup

```
cargo install cargo-insta
```

## Usage

When tests are executed, the output is compared against the snapshot. Incase they
differ, insta complains and offers to update the snapshot.

Snapshots are stored in `*.snap` files and might become outdated or stale when
tests are removed. `cargo insta test --unreferenced=delete` should remove unused files, but not always succeeds.
