# Release Process

This document describes how to prepare a Ripex release. It documents the
planned `v0.3.0` release workflow; it does not claim that `v0.3.0` has been
published.

## 1. Prepare the release

Before creating a tag:

1. Update `Cargo.toml` and `Cargo.lock` to the intended version (`0.3.0` for
   this release).
2. Add the release date and changes to `CHANGELOG.md`.
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

## 2. Create the `v0.3.0` tag

After the preparation checks pass:

```bash
git tag -a v0.3.0 -m "Release v0.3.0"
git push origin v0.3.0
```

Pushing a `v*` tag starts `.github/workflows/release.yml`. The workflow checks
the tag/package-version match, validates the locked dependency graph, builds
the CLI for Linux, macOS, and Windows, and creates the GitHub Release only
after every build succeeds.

## 3. GitHub Release artifacts

The workflow builds one release binary per target and uploads deterministic
archives with these names for `v0.3.0`:

```text
ripex-v0.3.0-x86_64-unknown-linux-gnu.tar.gz
ripex-v0.3.0-x86_64-apple-darwin.tar.gz
ripex-v0.3.0-x86_64-pc-windows-msvc.zip
```

Each archive has a matching `<archive>.sha256` file. The release job verifies
all three checksums and publishes a sorted aggregate `SHA256SUMS` file along
with the archives. The archive metadata is normalized (including timestamps
and ownership) so repeated packaging of the same binary has reproducible
archive metadata.

The release job uses GitHub's generated release notes and has the only
`contents: write` permission. Validation and build jobs have
`contents: read` only. No crates.io token or publishing step is configured in
the workflow.

## 4. crates.io sequencing

Crates.io publication is deliberately manual and follows inspection of the
GitHub Release artifacts. From the clean, reviewed release commit, first
perform a dry run and then publish intentionally:

```bash
cargo publish --locked --dry-run
cargo publish --locked
```

Do not use `cargo package --allow-dirty` for a release. If packaging is needed
for local inspection, preserve a clean checkout and use `cargo package --locked`
without `--allow-dirty`. The GitHub tag workflow itself only builds the CLI
and creates the GitHub Release; it never publishes to crates.io.

## 5. Rollback and revocation

If a published crate version needs to be revoked, yank that exact version:

```bash
cargo yank --vers 0.3.0
```

Then prepare and publish a patch release (for example, `0.3.1`) after fixing
the issue. Treat any GitHub Release deletion or replacement as a separate,
explicit GitHub operation; this workflow does not perform rollback actions.
