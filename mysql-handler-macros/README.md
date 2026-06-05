# mysql-handler-macros

Procedural macros for [`mysql-handler`](https://crates.io/crates/mysql-handler)
engine cdylibs. The single attribute macro `#[plugin]` generates the
plugin manifest statics and the panic-safe init wrapper mysqld expects
at `INSTALL PLUGIN` time.

This crate is normally not depended on directly — `mysql-handler`
re-exports the macro through its `prelude` module, so an engine crate
only needs `mysql-handler` in `[dependencies]`.

```rust
use mysql_handler::prelude::*;

#[plugin(
    name = "my_engine",
    description = "Custom storage engine",
    version = 0x0001,
    license = License::Gpl,
    author = "me",
)]
#[derive(Default)]
pub struct MyEngine;
```

Refer to the macro rustdoc on `mysql-handler` for the full argument
list and validation rules.

## License

GPL-2.0-only.
