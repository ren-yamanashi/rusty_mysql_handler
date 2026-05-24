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

//! Build script for the example storage engine (cdylib). Use this as a
//! reference for the linker setup downstream engine cdylibs need. The plugin
//! manifest data symbols live in `src/lib.rs` as `#[no_mangle] pub static`,
//! so the linker exports them via Rust's standard cdylib export path on every
//! target. The only OS-specific concern left here is letting mysqld supply
//! the unresolved C++ symbols at load time on macOS.

fn main() {
    let target = std::env::var("TARGET").expect("TARGET is always set by cargo");

    // `links = "ha_rusty_shim"` on the mysql-handler crate exports this env var.
    match std::env::var("DEP_HA_RUSTY_SHIM_STATICLIB_DIR") {
        Ok(dir) => {
            println!("cargo:rustc-link-search=native={dir}");
            println!("cargo:rustc-link-lib=static=ha_rusty_shim");
        }
        Err(_) => {
            println!(
                "cargo:warning=mysql-handler shim staticlib not produced — the resulting cdylib will be missing C++ shim symbols and cannot be loaded by mysqld. Set MYSQL_HANDLER_FROM_SOURCE=1 or MYSQL_HANDLER_ARCHIVE=<path> to enable shim linking."
            );
        }
    }

    if target.contains("apple") {
        println!("cargo:rustc-link-lib=dylib=c++");
        // mysqld provides the unresolved C++ symbols (handler base class
        // methods, THR_LOCK helpers, etc.) at load time.
        println!("cargo:rustc-link-arg=-Wl,-undefined,dynamic_lookup");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }
}
