<h1 align="center">Rusty MySQL Handler</h1>
<p align="center">Build MySQL storage engine plugins in Rust.</p>


> [!WARNING]
> This project is currently experimental. APIs are unstable, behaviour may change between releases, and it is not yet ready for production use.

## 📦 Installation

```bash
cargo add mysql-handler
```

Or in `Cargo.toml`:

```toml
[dependencies]
mysql-handler = "0.1"
```

## 🚀 Usage

Implement the `StorageEngine` trait in a `cdylib` crate, then `INSTALL PLUGIN` it into a running `mysql:8.4` server.

#### Prerequisites

- Rust 1.85+
- `mysql:8.4` (any install method — local, Homebrew, Docker, RDS)

#### 1. New cdylib crate

```toml
[package]
name = "my-engine"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
mysql-handler = "0.1"
```

#### 2. Implement StorageEngine

```rust
use mysql_handler::engine::{EngineError, EngineResult, StorageEngine};
use std::ffi::CStr;

pub struct MyEngine;

impl StorageEngine for MyEngine {
    fn table_type(&self) -> &'static CStr { c"MY_ENGINE" }
    fn table_flags(&self) -> u64 { 0 }
    fn index_flags(&self, _i: u32, _p: u32, _all: bool) -> u32 { 0 }

    fn create(&mut self, _name: &str) -> EngineResult { Ok(()) }
    fn open(&mut self, _name: &str, _mode: i32) -> EngineResult { Ok(()) }
    fn close(&mut self) -> EngineResult { Ok(()) }

    fn rnd_init(&mut self, _scan: bool) -> EngineResult { Ok(()) }
    fn rnd_next(&mut self, _buf: &mut [u8]) -> EngineResult {
        Err(EngineError::EndOfFile)
    }
    fn position(&mut self, _record: &[u8], _ref_out: &mut [u8]) {}
    fn rnd_pos(&mut self, _buf: &mut [u8], _pos: &[u8]) -> EngineResult { Ok(()) }

    fn info(&mut self, _flag: u32) -> EngineResult { Ok(()) }
}
```

Plugin manifest macro: copy from [`examples/engine/src/lib.rs`](./examples/engine/src/lib.rs).

#### 3. Build

```bash
MYSQL_HANDLER_FROM_SOURCE=1 cargo build --release
# or
MYSQL_HANDLER_ARCHIVE=/path/to/libha_rusty_shim.a.gz cargo build --release
```

#### 4. Install

```sql
INSTALL PLUGIN my_engine SONAME 'libmy_engine.so';
CREATE TABLE t (id INT) ENGINE=MY_ENGINE;
```

## 📊 Performance

Per-callback FFI overhead, callback profile, and (when filled) OLTP throughput live in [`tests/sysbench/RESULTS.md`](./tests/sysbench/RESULTS.md).

## ❗ Issue

If you have any questions or suggestions, please open an [issue](https://github.com/ren-yamanashi/rusty_mysql_handler/issues).

## ©️ License

[GPL-2.0](./LICENSE)
