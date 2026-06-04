// Copyright (C) 2026 ren-yamanashi
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License, version 2.0,
// as published by the Free Software Foundation.
//
// This program is designed to work with certain software (including
// but not limited to OpenSSL) that is licensed under separate terms,
// as designated in a particular file or component or in included license
// documentation. The authors of this program hereby grant you an additional
// permission to link the program and your derivative works with the
// separately licensed software that they have either included with
// the program or referenced in the documentation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program; if not, see <https://www.gnu.org/licenses/>.

//! Build-script helper for `mysql-handler` downstream engine cdylibs.
//!
//! A downstream engine crate's `build.rs` reduces to a single call to
//! [`configure`]. The helper wires the C++ shim staticlib and the
//! platform's C++ runtime into the link line so the resulting cdylib
//! is loadable by mysqld.

const MISSING_SHIM_WARNING: &str = "cargo:warning=mysql-handler shim staticlib not produced — the resulting cdylib will be missing C++ shim symbols and cannot be loaded by mysqld. Set MYSQL_HANDLER_FROM_SOURCE=1 or MYSQL_HANDLER_ARCHIVE=<path> to enable shim linking.";

/// Configure the link line for a `mysql-handler` engine cdylib.
///
/// Call this from `build.rs`. The helper performs three steps:
///
/// 1. **Shim staticlib.** When the `mysql-handler` crate's build script
///    produced `libha_rusty_shim.a`, it exports the output directory via
///    `DEP_HA_RUSTY_SHIM_STATICLIB_DIR` (Cargo's `links` metadata
///    bridge). The helper emits `rustc-link-search` and
///    `rustc-link-lib=static=ha_rusty_shim` so the cdylib picks the
///    staticlib up.
/// 2. **C++ runtime.** On Apple targets the helper links `c++`; on
///    Linux, `stdc++`. Other targets are skipped — engines targeting
///    them must wire their own runtime.
/// 3. **macOS dynamic lookup.** On Apple targets, the helper adds
///    `-Wl,-undefined,dynamic_lookup` so the C++ symbols defined inside
///    mysqld (the loading host) are resolved at plugin-load time rather
///    than at link time.
///
/// When `DEP_HA_RUSTY_SHIM_STATICLIB_DIR` is unset — that is, the user
/// has not opted into `MYSQL_HANDLER_FROM_SOURCE=1` or
/// `MYSQL_HANDLER_ARCHIVE=<path>` for the `mysql-handler` dependency —
/// the helper emits a `cargo:warning=` describing the env vars to set.
/// The build is not failed; `cargo check` and IDE workflows still work,
/// but the produced cdylib will not load into mysqld.
///
/// # Panics
///
/// Panics if Cargo did not export `TARGET`. Cargo always exports it
/// before invoking a build script, so this branch is unreachable in
/// practice.
pub fn configure() {
    let target = std::env::var("TARGET").expect("TARGET is always set by cargo");
    let shim_dir = std::env::var("DEP_HA_RUSTY_SHIM_STATICLIB_DIR").ok();

    for line in directives(&target, shim_dir.as_deref()) {
        println!("{line}");
    }
}

fn directives(target: &str, shim_dir: Option<&str>) -> Vec<String> {
    let mut out = Vec::new();
    match shim_dir {
        Some(dir) => {
            out.push(format!("cargo:rustc-link-search=native={dir}"));
            out.push("cargo:rustc-link-lib=static=ha_rusty_shim".into());
        }
        None => out.push(MISSING_SHIM_WARNING.into()),
    }

    if target.contains("apple") {
        out.push("cargo:rustc-link-lib=dylib=c++".into());
        out.push("cargo:rustc-link-arg=-Wl,-undefined,dynamic_lookup".into());
    } else if target.contains("linux") {
        out.push("cargo:rustc-link-lib=dylib=stdc++".into());
    }

    out
}

#[cfg(test)]
mod tests {
    use super::{MISSING_SHIM_WARNING, directives};

    #[test]
    fn apple_with_shim_dir_emits_static_cpp_and_dynamic_lookup() {
        let lines = directives("aarch64-apple-darwin", Some("/tmp/shim"));
        assert!(
            lines
                .iter()
                .any(|l| l == "cargo:rustc-link-search=native=/tmp/shim")
        );
        assert!(
            lines
                .iter()
                .any(|l| l == "cargo:rustc-link-lib=static=ha_rusty_shim")
        );
        assert!(lines.iter().any(|l| l == "cargo:rustc-link-lib=dylib=c++"));
        assert!(
            lines
                .iter()
                .any(|l| l == "cargo:rustc-link-arg=-Wl,-undefined,dynamic_lookup")
        );
        assert!(!lines.iter().any(|l| l.starts_with("cargo:warning=")));
    }

    #[test]
    fn linux_with_shim_dir_emits_static_and_stdcpp() {
        let lines = directives("x86_64-unknown-linux-gnu", Some("/var/cache/shim"));
        assert!(
            lines
                .iter()
                .any(|l| l == "cargo:rustc-link-search=native=/var/cache/shim")
        );
        assert!(
            lines
                .iter()
                .any(|l| l == "cargo:rustc-link-lib=static=ha_rusty_shim")
        );
        assert!(
            lines
                .iter()
                .any(|l| l == "cargo:rustc-link-lib=dylib=stdc++")
        );
        assert!(!lines.iter().any(|l| l.contains("dynamic_lookup")));
        assert!(!lines.iter().any(|l| l.starts_with("cargo:warning=")));
    }

    #[test]
    fn missing_shim_dir_emits_warning_and_no_static_link() {
        let lines = directives("aarch64-apple-darwin", None);
        assert!(lines.iter().any(|l| l == MISSING_SHIM_WARNING));
        assert!(
            !lines
                .iter()
                .any(|l| l.starts_with("cargo:rustc-link-search"))
        );
        assert!(!lines.iter().any(|l| l.contains("static=ha_rusty_shim")));
        // C++ runtime still wired so cargo check produces sensible output.
        assert!(lines.iter().any(|l| l == "cargo:rustc-link-lib=dylib=c++"));
    }

    #[test]
    fn unknown_target_skips_cpp_runtime() {
        let lines = directives("wasm32-unknown-unknown", Some("/x"));
        assert!(!lines.iter().any(|l| l.contains("c++")));
        assert!(!lines.iter().any(|l| l.contains("stdc++")));
        assert!(!lines.iter().any(|l| l.contains("dynamic_lookup")));
    }
}
