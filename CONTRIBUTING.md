# Contributing

## Prerequisites

- Rust stable (1.85+) — pinned via `rust-toolchain.toml`
- CMake 3.20+, C++20 compiler (clang 16+ / gcc 13+)
- MySQL 8.4 LTS source via the `mysql-server/` submodule

## Setup

```bash
git clone https://github.com/ren-yamanashi/rusty_mysql_handler.git
cd rusty_mysql_handler
make setup
```

`make help` lists every target. Typical PR loop: `make check` → `make test`
→ `make test_e2e`.

## Updating the E2E build base

`make test_e2e` compiles the plugin in Docker against a prebuilt
`mysql-server` tree published as a GitHub Release asset
(`mysql-base-<version>`). Recipe: `tests/e2e/Dockerfile.base`. Publisher:
`.github/workflows/publish-mysql-base.yml`.

To rebuild on a MySQL version bump:

1. Edit `Dockerfile.base` (new clone tag + commit SHA) and `MYSQL_VERSION`
   in the publisher workflow.
2. Run **Publish mysql build base** via `workflow_dispatch`; it uploads
   the tarball and prints the SHA-256.
3. Commit the 64-hex digest into `ARG MYSQL_BASE_SHA256=` in
   `tests/e2e/Dockerfile`.

The E2E build is fail-closed — it refuses to start if `MYSQL_BASE_SHA256`
does not match the downloaded asset.

## Releasing

Driven by [`release-plz`](https://release-plz.dev/): every main push
updates a rolling "Release" PR with the version bump (from Conventional
Commits) and changelog. Merging that PR tags, publishes to crates.io,
and creates the matching GitHub Release. The repo carries no
`CHANGELOG.md` — GitHub Releases is the canonical history.

The publish job runs in the **crates-io** GitHub environment. Configure
under **Settings → Environments**:

- Required reviewers (at least one maintainer)
- Deployment branch restriction = `main`
- Auth: trusted publishing (default; OIDC via
  `rust-lang/crates-io-auth-action`), or `CARGO_REGISTRY_TOKEN` as a
  fallback for the first publish

`pr-title-lint.yml` enforces Conventional Commits on PR titles so
release-plz can rely on them.

## Coding Conventions

- Rust edition 2024; `rustfmt` + `clippy` (lints in `Cargo.toml`)
- Every `extern "C"` callback wraps its body in `catch_unwind`
- FFI naming: `rust__handler__method` (C++ → Rust),
  `mysql__Class__method` (Rust → C++)
