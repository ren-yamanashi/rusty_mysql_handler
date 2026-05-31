# Contributing

## Prerequisites

- Rust stable (1.85+) — pinned via `rust-toolchain.toml`
- CMake 3.20+, C++20 compiler (clang 16+ / gcc 13+)
- MySQL 8.4 LTS source via the `mysql-server/` submodule

## Setup

Fork the repo on GitHub, clone your fork, then:

```bash
make setup
```

`make help` lists every target. Typical PR loop: `make check` → `make test`
→ `make test_e2e`.

## Updating the E2E build base

`make test_e2e` compiles the plugin in Docker against a prebuilt
`mysql-server` tree published as GitHub Release assets
(`mysql-base-<version>`), one per supported arch (x86_64, arm64). Recipe:
`tests/e2e/Dockerfile.base`. Publisher: `.github/workflows/publish-mysql-base.yml`.

To rebuild on a MySQL version bump:

1. Edit `Dockerfile.base` (new clone tag + commit SHA) and `MYSQL_VERSION`
   in the publisher workflow.
2. Run **Publish mysql build base** via `workflow_dispatch`; the matrix
   uploads `mysql-build-base-<version>.tar.gz` (x86_64, legacy name) and
   `mysql-build-base-<version>-arm64.tar.gz` and prints each SHA-256.
3. Commit the digests into `ARG MYSQL_BASE_SHA256_AMD64=` /
   `ARG MYSQL_BASE_SHA256_ARM64=` in `tests/e2e/Dockerfile`.

The E2E build is fail-closed — Docker refuses to start if the SHA for the
target arch is unset or does not match the downloaded asset. After the
first arm64 base publish, the arm64 SHA pin needs to be set or the
arm64 `Smoke` job will fail.

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
