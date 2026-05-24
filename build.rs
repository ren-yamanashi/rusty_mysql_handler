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

//! Build script.
//!
//! Three modes, mutually exclusive, picked by env var:
//! * `MYSQL_HANDLER_FROM_SOURCE=1` → cmake builds the shim staticlib locally.
//! * `MYSQL_HANDLER_ARCHIVE=<local path>` → gunzip the given `.a.gz` into
//!   `OUT_DIR/prebuilt/libha_rusty_shim.a`. The path must resolve on the
//!   local filesystem; this script does not perform network I/O.
//! * (default) → run bindgen only. Sufficient for `cargo check` /
//!   `cargo test`; the resulting `cdylib` is not loadable into mysqld.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn main() {
    BuildScript::new().run();
}

struct BuildScript {
    manifest_dir: String,
    mysql_src: String,
    mysql_build: String,
    out_dir: PathBuf,
}

impl BuildScript {
    fn new() -> Self {
        let manifest_dir =
            env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is always set by cargo");
        let mysql_src = match env::var("MYSQL_SOURCE_DIR") {
            Ok(v) => v,
            Err(_) => format!("{manifest_dir}/mysql-server"),
        };
        let mysql_build = match env::var("MYSQL_BUILD_DIR") {
            Ok(v) => v,
            Err(_) => format!("{manifest_dir}/build/mysql"),
        };
        let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is always set by cargo"));
        Self {
            manifest_dir,
            mysql_src,
            mysql_build,
            out_dir,
        }
    }

    fn run(&self) {
        self.run_bindgen();

        println!("cargo:rerun-if-env-changed=MYSQL_HANDLER_FROM_SOURCE");
        println!("cargo:rerun-if-env-changed=MYSQL_HANDLER_ARCHIVE");

        match env::var("MYSQL_HANDLER_FROM_SOURCE") {
            Ok(_) => self.build_shim_from_source(),
            Err(_) => {
                if let Ok(archive) = env::var("MYSQL_HANDLER_ARCHIVE") {
                    self.install_prebuilt_shim(Path::new(&archive));
                }
            }
        }
    }

    fn run_bindgen(&self) {
        let header = format!("{}/shim/bindgen_input.h", self.manifest_dir);

        println!("cargo:rerun-if-changed={header}");
        println!("cargo:rerun-if-env-changed=MYSQL_SOURCE_DIR");
        println!("cargo:rerun-if-env-changed=MYSQL_BUILD_DIR");

        let bindings = bindgen::Builder::default()
            .header(&header)
            .clang_arg("-x")
            .clang_arg("c++")
            .clang_arg("-std=c++20")
            .clang_arg(format!("-I{}/include", self.mysql_build))
            .clang_arg(format!("-I{}/include", self.mysql_src))
            .allowlist_var("HA_ERR_.*")
            .allowlist_var("HA_BINLOG_.*")
            .allowlist_type("Table_flags")
            .allowlist_type("thr_lock_type")
            .default_enum_style(bindgen::EnumVariation::Rust {
                non_exhaustive: false,
            })
            .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
            .generate()
            .expect("bindgen generation failed");

        let out_path = self.out_dir.join("sys_bindings.rs");
        bindings
            .write_to_file(&out_path)
            .expect("write bindgen output");
    }

    fn build_shim_from_source(&self) {
        for src in [
            "shim/binding.cc",
            "shim/binding.hpp",
            "shim/plugin.cc",
            "shim/CMakeLists.txt",
            "shim/rust_callbacks.hpp",
        ] {
            println!("cargo:rerun-if-changed={}/{src}", self.manifest_dir);
        }

        let dst = cmake::Config::new(format!("{}/shim", self.manifest_dir))
            .define("MYSQL_SOURCE_DIR", &self.mysql_src)
            .define("MYSQL_BUILD_DIR", &self.mysql_build)
            .build_target("ha_rusty_shim")
            .build();

        Self::publish_staticlib_dir(&format!("{}/build", dst.display()));
    }

    fn install_prebuilt_shim(&self, archive: &Path) {
        assert!(
            archive.is_file(),
            "MYSQL_HANDLER_ARCHIVE must point to a readable .a.gz file: {}",
            archive.display()
        );
        let prebuilt_dir = self.out_dir.join("prebuilt");
        fs::create_dir_all(&prebuilt_dir).expect("create prebuilt staticlib dir under OUT_DIR");
        let staticlib_path = prebuilt_dir.join("libha_rusty_shim.a");
        Self::extract_gz(archive, &staticlib_path);
        Self::publish_staticlib_dir(&prebuilt_dir.display().to_string());
    }

    fn extract_gz(gz_path: &Path, dest: &Path) {
        let gz_file = fs::File::open(gz_path).expect("open MYSQL_HANDLER_ARCHIVE for reading");
        let mut decoder = flate2::read::GzDecoder::new(io::BufReader::new(gz_file));
        let mut out_file =
            fs::File::create(dest).expect("create extracted staticlib under OUT_DIR/prebuilt");
        io::copy(&mut decoder, &mut out_file).expect("decompress MYSQL_HANDLER_ARCHIVE");
    }

    // `links = "ha_rusty_shim"` exports this as `DEP_HA_RUSTY_SHIM_STATICLIB_DIR`
    // to dependent build scripts. The cdylib-side link directives live there
    // because rlib build-scripts cannot emit `+whole-archive` modifiers or
    // `rustc-link-arg-cdylib` that survive the hop to a dependent cdylib.
    fn publish_staticlib_dir(dir: &str) {
        println!("cargo:staticlib-dir={dir}");
    }
}
