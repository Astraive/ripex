# Release Process

This document outlines the release process for Ripex.

## 1. Pre-release Checks
Before cutting a release:
- Ensure `cargo test --all-targets --all-features` passes.
- Check formatting: `cargo fmt --all -- --check`.
- Check clippy: `cargo clippy --all-targets --all-features -- -D warnings`.
- Ensure MSRV compliance: compile with toolchain matching `rust-version` in `Cargo.toml`.
- Update `CHANGELOG.md` with the new version and release date.

## 2. Creating a Release Tag
1. Tag the release:
   ```bash
   git tag -a v0.2.0 -m "Release v0.2.0"
   ```
2. Push the tags:
   ```bash
   git push origin v0.2.0
   ```

## 3. Distribution Policy
- **crates.io**: Publish via `cargo publish`. Only release clean working trees that pass CI.
- **GitHub Releases**: Post artifacts, checksums, and release notes for each tagged release.

## 4. Rollback and Revocation
- In the event of a critical issue post-release, yank the crate version on crates.io:
  ```bash
  cargo yank --vers 0.2.0
  ```
- Publish a patch release (e.g., `0.2.1`) to resolve the issue.
