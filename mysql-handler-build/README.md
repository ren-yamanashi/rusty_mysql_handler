# mysql-handler-build

Build-script helper for [`mysql-handler`](https://crates.io/crates/mysql-handler)
downstream engine cdylibs. Drop a single call into your `build.rs`:

```rust
fn main() {
    mysql_handler_build::configure();
}
```

`configure()` wires the C++ shim staticlib (via the
`DEP_HA_RUSTY_SHIM_STATICLIB_DIR` env var exported by `mysql-handler`'s
own build script) and the platform's C++ runtime into the link line.
Refer to the crate rustdoc for the env vars consulted and the warning
behaviour when the shim staticlib is unavailable.

## License

GPL-2.0-only.
