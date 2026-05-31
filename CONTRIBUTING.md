# Contributing

## Prerequisites

- Rust stable (1.85+) — automatically configured via `rust-toolchain.toml`
- CMake 3.20+
- C++20 compiler (clang 16+ / gcc 13+)
- MySQL 8.4 LTS source (included as a submodule in `mysql-server/`)

## Project Structure

```sh
mysql-handler/
├── src/
│   └── lib.rs
├── examples/
│   └── engine/             # Example engine (staticlib)
├── mysql-server/           # MySQL 8.4 LTS source (submodule)
```

## Setup

```bash
git clone https://github.com/ren-yamanashi/rusty_mysql_handler.git
cd rusty_mysql_handler
make setup
```

## Commands

```bash
cargo check --workspace    # Type check
cargo build --release      # Build
make fmt                   # Format all Rust code (rustfmt)
make lint                  # Run clippy (treats warnings as errors)
make test                  # Run all tests
make check                 # cargo check + clippy + fmt check
```

Run `make help` to see all available targets.

## Updating the E2E build base

The E2E smoke test (`make test_e2e` and the E2E workflow) compiles the plugin
inside Docker against a prebuilt, cmake-configured `mysql-server` tree that is
published as a GitHub Release asset (`mysql-base-<version>`), rather than
cloning and configuring MySQL on every run. See `tests/e2e/Dockerfile.base`
(the recipe) and `.github/workflows/publish-mysql-base.yml` (the publisher).

This base changes only on a MySQL version bump or an edit to `Dockerfile.base`.
To update it:

1. Edit `tests/e2e/Dockerfile.base` (e.g. bump the `mysql-8.4.x` clone tag and
   the pinned commit SHA) and `MYSQL_VERSION` in
   `.github/workflows/publish-mysql-base.yml`.
2. Run the **Publish mysql build base** workflow from the Actions tab
   (`workflow_dispatch`). It builds the tree, uploads `mysql-build-base-<version>.tar.gz`
   plus its `.sha256` to the `mysql-base-<version>` release, and prints the
   SHA-256 digest in the run log.
3. Commit that digest — the 64-hex value only, not the whole `<hash>  <file>`
   line — into `ARG MYSQL_BASE_SHA256=` in `tests/e2e/Dockerfile` (and update
   `ARG MYSQL_BASE_URL` if the version changed).

Publishing is manual (`workflow_dispatch`) on purpose: each run re-tars the
tree, so the asset checksum changes and the in-repo pin must be updated by
hand. The E2E build is fail-closed — it refuses to build when
`MYSQL_BASE_SHA256` is unset or does not match the downloaded asset.

## Releasing

A `vX.Y.Z` tag push triggers `publish-rust.yml` (cargo publish) followed by
`github-release.yml` (auto-generated changelog). The publish job runs in the
`crates-io` GitHub environment — for that environment to be a real security
boundary rather than decoration, configure it under
**Settings → Environments → crates-io** with:

1. **Required reviewers** — at least one maintainer must approve each
   deployment before `cargo publish` runs.
2. **Deployment branch and tag restriction** — limit deployments to tags
   matching `v[0-9]+.[0-9]+.[0-9]+*` so a workflow_dispatch from a
   feature branch cannot publish.
3. **Authentication** — either:
   - **Trusted publishing (recommended)**: configure `mysql-handler` on
     crates.io to trust this repository's `publish-rust.yml`. The default
     tag-push path uses OIDC and needs no secret.
   - **Token fallback**: store `CARGO_REGISTRY_TOKEN` as an environment
     secret. Used by the `workflow_dispatch` path with
     `auth_mode = token`, kept for the first publish (before trusted
     publishing can be configured) and disaster recovery.

`release-dry-run.yml` runs the same gates on every main push so a
release-blocking issue (license, lint, test, dry-run publish) is caught
before tagging.

PR labels drive `gh release create --generate-notes` categorisation (see
`.github/release.yml`). Labels are applied manually before the tag is
pushed — set the matching Conventional Commits label (`feat`, `fix`,
`refactor`, `docs`, `build`, `ci`, `chore`, `test`, `breaking`) on each
PR merged into the release.

## Coding Conventions

- Rust edition 2024. Follow `rustfmt` and `clippy` (lint rules in `Cargo.toml`)
- All `extern "C"` callbacks must use `catch_unwind` (panics abort MySQL)
- FFI naming: `rust__handler__method` (C++ calls Rust), `mysql__Class__method` (Rust calls C++)
