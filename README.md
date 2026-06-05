<h1 align="center">Rusty MySQL Handler</h1>
<p align="center">Build MySQL storage engine plugins in Rust.</p>


> [!WARNING]
> This project is currently experimental. APIs are unstable, behaviour may change between releases, and it is not yet ready for production use.

## 📦 Installation

```bash
cargo add mysql-handler
cargo add --build mysql-handler-build
```

Or in `Cargo.toml`:

```toml
[dependencies]
mysql-handler = "0.2"

[build-dependencies]
mysql-handler-build = "0.2"
```

`mysql-handler` re-exports the `#[plugin]` attribute macro from `mysql-handler-macros`; depend on the macro crate transitively rather than directly.

## 🚀 Quick start

A loadable engine cdylib fits in about thirty lines of user code: a `Cargo.toml`, a one-line `build.rs`, and an engine struct that implements `StorageEngine` (plus `IndexedEngine` when the engine serves indexes). The `#[plugin]` macro on the engine struct emits the plugin manifest, the panic-safe init entry point, and the `EngineCapabilities` impl for every listed capability.

#### Prerequisites

- Rust 1.85+
- `mysql:8.4` (any install method — local, Homebrew, Docker, RDS)
- One of `MYSQL_HANDLER_FROM_SOURCE=1` (cmake-builds the C++ shim from the bundled `mysql-server/` submodule) or `MYSQL_HANDLER_ARCHIVE=<path>` (uses a prebuilt `libha_rusty_shim.a.gz`) when producing the final cdylib

#### 1. New cdylib crate

```toml
[package]
name = "my-engine"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
mysql-handler = "0.2"

[build-dependencies]
mysql-handler-build = "0.2"
```

#### 2. `build.rs`

```rust
fn main() {
    mysql_handler_build::configure();
}
```

#### 3. `src/lib.rs`

```rust
use mysql_handler::prelude::*;
use std::ffi::CStr;

#[plugin(
    name = "MY_ENGINE",
    description = "Custom storage engine",
    version = 0x0001,
    license = License::Gpl,
    author = "you",
    capabilities = [Indexed],
)]
#[derive(Debug, Default)]
pub struct MyEngine;

impl StorageEngine for MyEngine {
    fn table_type(&self) -> &'static CStr { c"MY_ENGINE" }
    fn table_flags(&self) -> u64 { 0 }

    fn create(&mut self, _name: &str, _table_def: Option<&mysql_handler::sys::DdTable>) -> EngineResult { Ok(()) }
    fn open(&mut self, _name: &str, _mode: i32, _table_def: Option<&mysql_handler::sys::DdTable>) -> EngineResult { Ok(()) }
    fn close(&mut self) -> EngineResult { Ok(()) }

    fn rnd_init(&mut self, _scan: bool) -> EngineResult { Ok(()) }
    fn rnd_next(&mut self, _buf: &mut [u8]) -> EngineResult {
        Err(EngineError::EndOfFile)
    }
    fn rnd_pos(&mut self, _buf: &mut [u8], _pos: &[u8]) -> EngineResult { Ok(()) }
    fn position(&mut self, _record: &[u8], _ref_out: &mut [u8]) {}

    fn info(&mut self, _flag: u32) -> EngineResult { Ok(()) }
}

impl IndexedEngine for MyEngine {}
```

Each `#[plugin]` argument maps directly onto the MySQL plugin manifest. `capabilities = [Indexed]` declares the optional [`IndexedEngine`] sub-trait the engine opts into; omit it for engines without index support. See [`examples/engine/`](./examples/engine/) for a reference implementation that exercises every layer.

#### 4. Build

```bash
MYSQL_HANDLER_FROM_SOURCE=1 cargo build --release
# or
MYSQL_HANDLER_ARCHIVE=/path/to/libha_rusty_shim.a.gz cargo build --release
```

#### 5. Install

```sql
INSTALL PLUGIN my_engine SONAME 'libmy_engine.so';
CREATE TABLE t (id INT) ENGINE=MY_ENGINE;
```

## 🔁 Migrating from 0.1.0

0.2.0 is a breaking change: the plugin manifest moves into the `#[plugin]` attribute macro on the engine struct, the legacy `plugin_manifest.rs` boilerplate is gone, and index-related virtual methods now live on a separate `IndexedEngine` sub-trait reached through the `EngineCapabilities` dispatcher. 0.1.0 stays on crates.io for archival reference; new engines should target 0.2 from the start.

## 📊 Performance

Per-callback FFI overhead, callback profile, and OLTP throughput live in [`tests/sysbench/RESULTS.md`](./tests/sysbench/RESULTS.md).

## ❗ Issue

If you have any questions or suggestions, please open an [issue](https://github.com/ren-yamanashi/rusty_mysql_handler/issues).

## ©️ License

[GPL-2.0](./LICENSE)
