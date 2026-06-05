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

//! Procedural macros for `mysql-handler` engine cdylibs.
//!
//! The single macro this crate exposes, [`plugin`], generates the
//! three `#[unsafe(no_mangle)] pub static` items mysqld dlsyms at
//! `INSTALL PLUGIN` time, plus the panic-safe init wrapper that
//! registers the engine factory.

// `pub(crate)` on internal items is required for `unreachable_pub`;
// the resulting `redundant_pub_crate` lint is a false positive here.
#![allow(clippy::redundant_pub_crate)]

mod args;
mod capability;
mod expand;

use proc_macro::TokenStream;
use syn::{ItemStruct, parse_macro_input};

use crate::args::PluginArgs;

/// Attribute macro that turns a `Default`-constructible engine struct
/// into a loadable MySQL plugin.
///
/// # Arguments
///
/// - `name`: plugin name (string literal, ASCII, ≤ 64 bytes, no NUL).
///   SQL identifies the engine by this name in `INSTALL PLUGIN <name>`.
/// - `description`: human-readable description (string literal).
/// - `version`: plugin version, any const expression evaluating to
///   `c_uint` (commonly `env!("CARGO_PKG_VERSION")` parsed elsewhere
///   or a literal like `0x0001`).
/// - `license`: any const expression of type
///   [`mysql_handler::license::License`]. The discriminant is folded
///   into the static initialiser at compile time.
/// - `author`: author or organisation name (string literal).
/// - `capabilities` (optional): bracketed list of sub-trait
///   discriminants the engine opts into. Accepted values: `Indexed`.
///   Each entry emits an `as_*` override on the generated
///   `EngineCapabilities` impl, so the engine must also implement the
///   matching sub-trait. Defaults to `[]` (engine declares no
///   capabilities). Additional capability identifiers are added when
///   their sub-traits ship with non-empty method sets.
/// - `handlerton` (optional): path to a `Default`-constructible
///   handlerton struct (typically a unit struct) implementing
///   [`mysql_handler::hton::Handlerton`]. When supplied the generated
///   `rust__plugin_init` additionally registers the handlerton so
///   engine-level callbacks (transactions, savepoints, discovery)
///   route through it. Omit when the engine only needs the per-handler
///   surface.
///
/// # Example
///
/// ```ignore
/// use mysql_handler::prelude::*;
///
/// #[plugin(
///     name = "my_engine",
///     description = "Custom storage engine",
///     version = 0x0001,
///     license = License::Gpl,
///     author = "me",
///     capabilities = [Indexed],
/// )]
/// #[derive(Default)]
/// pub struct MyEngine;
///
/// impl mysql_handler::engine::StorageEngine for MyEngine {}
/// impl mysql_handler::engine::IndexedEngine for MyEngine {}
/// ```
#[proc_macro_attribute]
pub fn plugin(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as PluginArgs);
    let target = parse_macro_input!(item as ItemStruct);
    expand::plugin(&args, &target)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
