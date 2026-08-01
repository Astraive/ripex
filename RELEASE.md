# Release Process

This document records the shipped Ripex release and provides reusable steps
for future releases.

## Published release: Ripex v0.3.0

Ripex v0.3.0 was published on 2026-08-01:

- [crates.io package](https://crates.io/crates/ripex/0.3.0)
- [docs.rs API documentation](https://docs.rs/ripex/0.3.0)
- [GitHub release](https://github.com/Astraive/ripex/releases/tag/v0.3.0)

Both `.github/workflows/ci.yml` and `.github/workflows/release.yml` passed
after the Graxus dependency migration. The release workflow validated the
locked dependency graph, built the CLI for Linux, macOS, and Windows, and
published the GitHub release only after every build succeeded.

The downstream Graxus workspace consumes the published `ripex` 0.3.0 package;
this record does not assert a separate Graxus release.

The published release contains these deterministic archives:

```text
ripex-v0.3.0-x86_64-unknown-linux-gnu.tar.gz
ripex-v0.3.0-x86_64-apple-darwin.tar.gz
ripex-v0.3.0-x86_64-pc-windows-msvc.zip
```

Each archive has a matching `<archive>.sha256` file and the release includes
the sorted aggregate `SHA256SUMS` file. Archive metadata is normalized,
including timestamps and ownership, so repeated packaging of the same binary
has reproducible archive metadata.

Crates.io publication for this release was a deliberate manual step after
inspection of the GitHub Release artifacts. The tag workflow itself only
validates and builds the release artifacts; it does not publish to crates.io.

## Reusable future-release checklist

### 1. Prepare the release

Before creating a tag:

1. Update `Cargo.toml` and `Cargo.lock` to the intended version
   (`<VERSION>`).
2. Add the release date, links, and changes to `CHANGELOG.md`.
3. Work from a clean checkout. The tag workflow rejects a dirty checkout and
   requires the tag to match the package version exactly.
4. Run the same locked checks used by the workflow:

   ```bash
   cargo fmt --all -- --check
   cargo check --locked --lib
   cargo check --locked --lib --no-default-features --features "lang-all"
   cargo test --locked --all-targets --all-features
   cargo clippy --locked --all-targets --all-features -- -D warnings
   ```

The library check with no feature override uses the package's default feature
set. The release build intentionally does not rely on those defaults: it
enables the CLI explicitly with `--no-default-features --features
"cli,lang-all"`.

### 2. Tag and build the release

After the preparation checks pass:

```bash
git tag -a v<VERSION> -m "Release v<VERSION>"
git push origin v<VERSION>
```

Pushing a `v*` tag starts `.github/workflows/release.yml`. The workflow checks
the tag/package-version match, validates the locked dependency graph, builds
the CLI for Linux, macOS, and Windows, and creates the GitHub Release only
after every build succeeds.

### 3. Verify GitHub release artifacts

The workflow builds one release binary per target and uploads deterministic
archives named with the version and target, for example:

```text
ripex-v<VERSION>-x86_64-unknown-linux-gnu.tar.gz
ripex-v<VERSION>-x86_64-apple-darwin.tar.gz
ripex-v<VERSION>-x86_64-pc-windows-msvc.zip
```

Each archive must have a matching `<archive>.sha256` file. The release job
verifies every checksum and publishes a sorted aggregate `SHA256SUMS` file.
The release job uses GitHub's generated release notes and has the only
`contents: write` permission; validation and build jobs have `contents: read`
only.

### 4. Publish to crates.io

After inspecting the GitHub Release artifacts from the clean, reviewed
release commit, perform a dry run and then publish intentionally:

```bash
cargo publish --locked --dry-run
cargo publish --locked
```

Do not use `cargo package --allow-dirty` for a release. If packaging is needed
for local inspection, preserve a clean checkout and use
`cargo package --locked` without `--allow-dirty`.

### 5. Roll back or revoke a release

If a published crate version needs to be revoked, yank that exact version:

```bash
cargo yank --vers <VERSION>
```

Then prepare and publish a patch release (for example, `<VERSION>` with an
incremented patch component) after fixing the issue. Treat any GitHub Release
deletion or replacement as a separate, explicit GitHub operation; the
workflow does not perform rollback actions.
