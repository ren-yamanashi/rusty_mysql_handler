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

//! `NoopEngine`: minimal `StorageEngine` impl used by per-callback
//! microbenchmarks. Every benched method returns `Ok(())` (or writes
//! one byte where the FFI shape requires a row buffer write) so the
//! measurement isolates FFI dispatch cost rather than the cost of any
//! real storage logic.

// `unreachable_pub` fires on bench sub-modules; see common/mod.rs.
#![allow(unreachable_pub)]

use std::ffi::CStr;

use mysql_handler::engine::{EngineResult, RKeyFunction, StorageEngine};
use mysql_handler::sys;

#[derive(Debug, Default)]
pub struct NoopEngine;

impl NoopEngine {
    pub fn new() -> Self {
        Self
    }
}

impl StorageEngine for NoopEngine {
    fn table_type(&self) -> &'static CStr {
        c"NOOPBENCH"
    }

    fn table_flags(&self) -> u64 {
        0
    }

    fn index_flags(&self, _idx: u32, _part: u32, _all_parts: bool) -> u32 {
        0
    }

    fn create(&mut self, _name: &str, _table_def: Option<&sys::DdTable>) -> EngineResult {
        Ok(())
    }

    fn open(&mut self, _name: &str, _mode: i32, _table_def: Option<&sys::DdTable>) -> EngineResult {
        Ok(())
    }

    fn close(&mut self) -> EngineResult {
        Ok(())
    }

    fn rnd_init(&mut self, _scan: bool) -> EngineResult {
        Ok(())
    }

    fn rnd_next(&mut self, buf: &mut [u8]) -> EngineResult {
        if let Some(first) = buf.first_mut() {
            *first = 0;
        }
        Ok(())
    }

    fn rnd_pos(&mut self, buf: &mut [u8], _pos: &[u8]) -> EngineResult {
        if let Some(first) = buf.first_mut() {
            *first = 0;
        }
        Ok(())
    }

    fn position(&mut self, _record: &[u8], ref_out: &mut [u8]) {
        if let Some(first) = ref_out.first_mut() {
            *first = 0;
        }
    }

    fn info(&mut self, _flag: u32) -> EngineResult {
        Ok(())
    }

    fn write_row(&mut self, _buf: &[u8]) -> EngineResult {
        Ok(())
    }

    fn update_row(&mut self, _old: &[u8], _new: &[u8]) -> EngineResult {
        Ok(())
    }

    fn delete_row(&mut self, _buf: &[u8]) -> EngineResult {
        Ok(())
    }

    fn index_init(&mut self, _idx: u32, _sorted: bool) -> EngineResult {
        Ok(())
    }

    fn index_read_map(
        &mut self,
        buf: &mut [u8],
        _key: &[u8],
        _find_flag: RKeyFunction,
    ) -> EngineResult {
        if let Some(first) = buf.first_mut() {
            *first = 0;
        }
        Ok(())
    }

    fn index_next(&mut self, buf: &mut [u8]) -> EngineResult {
        if let Some(first) = buf.first_mut() {
            *first = 0;
        }
        Ok(())
    }
}
