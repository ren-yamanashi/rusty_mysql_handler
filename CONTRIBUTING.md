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

## Coding Conventions

- Rust edition 2024. Follow `rustfmt` and `clippy` (lint rules in `Cargo.toml`)
- All `extern "C"` callbacks must use `catch_unwind` (panics abort MySQL)
- FFI naming: `rust__handler__method` (C++ calls Rust), `mysql__Class__method` (Rust calls C++)
