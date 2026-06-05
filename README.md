<h1 align="center">Rusty MySQL Handler</h1>
<p align="center">Build MySQL storage engine plugins in Rust.</p>


> [!WARNING]
> Experimental. APIs are unstable and not yet production-ready.

## 📦 Installation

```toml
[dependencies]
mysql-handler = "0.2"

[build-dependencies]
mysql-handler-build = "0.2"
```

## 🚀 Quick start

#### Prerequisites

- Rust 1.85+
- `mysql:8.4`

#### 1. `Cargo.toml`

```toml
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
)]
#[derive(Debug, Default)]
pub struct MyEngine;

impl StorageEngine for MyEngine {
    fn table_type(&self) -> &'static CStr { c"MY_ENGINE" }
    fn table_flags(&self) -> u64 { 0 }
    fn index_flags(&self, _idx: u32, _part: u32, _all_parts: bool) -> u32 { 0 }

    fn create(&mut self, _name: &str, _td: Option<&mysql_handler::sys::DdTable>) -> EngineResult { Ok(()) }
    fn open(&mut self, _name: &str, _mode: i32, _td: Option<&mysql_handler::sys::DdTable>) -> EngineResult { Ok(()) }
    fn close(&mut self) -> EngineResult { Ok(()) }

    fn rnd_init(&mut self, _scan: bool) -> EngineResult { Ok(()) }
    fn rnd_next(&mut self, _buf: &mut [u8]) -> EngineResult { Err(EngineError::EndOfFile) }
    fn rnd_pos(&mut self, _buf: &mut [u8], _pos: &[u8]) -> EngineResult { Ok(()) }
    fn position(&mut self, _record: &[u8], _ref_out: &mut [u8]) {}

    fn info(&mut self, _flag: u32) -> EngineResult { Ok(()) }
}
```

A full reference engine lives in [`examples/engine/`](./examples/engine/).

#### 4. Build

`build.rs` needs the C++ shim staticlib (`libha_rusty_shim.a`):

```bash
MYSQL_HANDLER_FROM_SOURCE=1 cargo build --release   # cmake from mysql-server/ submodule
MYSQL_HANDLER_ARCHIVE=<path> cargo build --release  # prebuilt .a.gz
```

#### 5. Install

```sql
INSTALL PLUGIN my_engine SONAME 'libmy_engine.so';
CREATE TABLE t (id INT) ENGINE=MY_ENGINE;
```

## 📊 Performance

[`tests/sysbench/RESULTS.md`](./tests/sysbench/RESULTS.md).

## ❗ Issue

[Open an issue](https://github.com/ren-yamanashi/rusty_mysql_handler/issues).

## ©️ License

[GPL-2.0](./LICENSE)
