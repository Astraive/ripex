# Release Process

This document records the shipped Ripex release and provides reusable steps
for future releases.

## Release: Ripex v0.4.0

Ripex v0.4.0 publishes universal comment retention and docstring extraction across Cargo, NPM, and PyPI:

- **Rust Crate**: `cargo add ripex` · [crates.io](https://crates.io/crates/ripex/0.4.0) · [docs.rs](https://docs.rs/ripex/0.4.0)
- **Node.js Package**: `npm install @astraive/ripex` · [npmjs.com](https://www.npmjs.com/package/@astraive/ripex)
- **Python Package**: `pip install ripex` · [pypi.org](https://pypi.org/project/ripex/)
- **GitHub Release**: [GitHub release](https://github.com/Astraive/ripex/releases/tag/v0.4.0)

## Historical release: Ripex v0.3.1
Ripex v0.3.1 was published on 2026-10-16 with initial PyO3 Python bindings and Node toolchain fixes.

## Historical release: Ripex v0.3.0
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

Pushing a `v*` tag triggers the four specialized release workflows:

1. `.github/workflows/release.yml`: Validates, builds CLI archives for Linux, macOS, and Windows, and publishes the GitHub Release.
2. `.github/workflows/cargo-release.yml`: Validates the Rust package and publishes `ripex` to Crates.io when `CARGO_REGISTRY_TOKEN` is configured.
3. `.github/workflows/node-release.yml`: Builds native binaries and assembles the 5 platform packages and root `@astraive/ripex` package for npm.
4. `.github/workflows/python-release.yml`: Builds sdist and cross-platform abi3 wheels for PyPI.
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

Crates.io publication is automated by `.github/workflows/cargo-release.yml` upon tag push when the `CARGO_REGISTRY_TOKEN` secret is configured in repository settings.

Manual fallback:
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
