# mysql-handler

Rust bindings for the MySQL 8.4 storage engine handler API. Write a
storage engine plugin in Rust by implementing the [`StorageEngine`]
trait, decorating the type with the [`plugin`] attribute macro, and
loading the resulting cdylib with `INSTALL PLUGIN`.

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
    /* ... open, close, rnd_init, rnd_next, info, etc. */
}
```

Pair this crate with [`mysql-handler-build`](https://crates.io/crates/mysql-handler-build)
in `build-dependencies` and call `mysql_handler_build::configure()`
from `build.rs` to wire the C++ shim staticlib into the link line. See
the rustdoc on `StorageEngine` and `plugin` for the full callback
surface and macro arguments.

A reference engine that exercises every binding lives in
[`examples/engine/`](https://github.com/ren-yamanashi/rusty_mysql_handler/tree/main/examples/engine).

## License

GPL-2.0-only.
