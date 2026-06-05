# Contributing

## Prerequisites

- Rust stable (1.85+) — pinned via `rust-toolchain.toml`
- CMake 3.20+, C++20 compiler (clang 16+ / gcc 13+)
- MySQL 8.4 LTS source via the `mysql-server/` submodule

## Setup

Fork and clone the repo, then:

```bash
make setup
```

`make help` lists every target. Typical PR loop: `make check` → `make test`.
End-to-end plugin-load coverage runs in CI (Smoke job in
`.github/workflows/e2e.yml`); there is no local Make wrapper for it.

## Workspace layout

Three crates ship out of this repository:

- `mysql-handler` (`src/`) — the runtime crate. Hosts the
  `StorageEngine` / `IndexedEngine` / `EngineCapabilities` traits, the
  `ffi_boundary()` `catch_unwind` wrappers, one Rust callback per
  handler / handlerton virtual, and the bindgen output (`sys.rs`).
- `mysql-handler-build` (`mysql-handler-build/`) — a zero-dependency
  build-script helper downstream cdylibs invoke from their `build.rs`.
- `mysql-handler-macros` (`mysql-handler-macros/`) — the `#[plugin]`
  proc-macro crate. Re-exported through `mysql_handler::prelude` so
  downstream depends on it transitively.

`shim/` is the C++ staticlib (`libha_rusty_shim.a`) that subclasses
`handler` and forwards each virtual to a `rust__handler__*` callback.
`examples/engine/` is a downstream cdylib used as a reference engine and
as the e2e smoke target; it consumes all three workspace crates exactly
the way an external user would.

## Updating the E2E build base

The Smoke CI job compiles the plugin in Docker against a prebuilt
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

## Coding Conventions

- Rust edition 2024; `rustfmt` + `clippy` (lints in `Cargo.toml`)
- Every `extern "C"` callback wraps its body in `catch_unwind`
- FFI naming: `rust__handler__method` (C++ → Rust),
  `mysql__Class__method` (Rust → C++)
- PR titles follow Conventional Commits (enforced by `pr-title-lint.yml`)
