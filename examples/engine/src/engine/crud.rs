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

//! Non-transactional update / delete helpers for [`TrivialEngine`].
//! The shim routes these only when no transactional handlerton is
//! attached; with a transaction the [`crate::trivial_txn::TrivialTxn`]
//! op log replaces them.

use mysql_handler::engine::{EngineError, EngineResult};

use super::TrivialEngine;
use crate::store;

impl TrivialEngine {
    /// Apply a non-transactional row update. When the table has a
    /// schema, key the lookup; otherwise fall back to byte-equality
    /// against the committed pre-image.
    pub(super) fn apply_update(&mut self, old: &[u8], new: &[u8]) -> EngineResult {
        let meta = match self.meta.as_ref() {
            Some(m) => m,
            None => {
                return finish_replace(store::replace_by_bytes(&self.table, old, new.to_vec()));
            }
        };
        let old_key = store::extract_key_from_row(old, meta);
        let new_key = store::extract_key_from_row(new, meta);
        match (old_key, new_key) {
            (Some(k_old), Some(k_new)) if k_old == k_new => {
                finish_replace(store::replace_by_key(&self.table, &k_old, new.to_vec()))
            }
            (Some(k_old), Some(k_new)) => {
                // Indexed column changed: drop the old entry and
                // reinsert under the new key so a later
                // `WHERE id = new` finds it.
                if !store::remove_by_key(&self.table, &k_old) {
                    return Err(EngineError::EndOfFile);
                }
                store::commit_keyed(&self.table, vec![(k_new, new.to_vec())]);
                Ok(())
            }
            _ => finish_replace(store::replace_by_bytes(&self.table, old, new.to_vec())),
        }
    }

    /// Apply a non-transactional row deletion. Like
    /// [`Self::apply_update`], picks the keyed path when meta is
    /// registered and falls back to byte equality otherwise.
    pub(super) fn apply_delete(&mut self, buf: &[u8]) -> EngineResult {
        let key = match self.meta.as_ref() {
            Some(m) => store::extract_key_from_row(buf, m),
            None => None,
        };
        let removed = match key {
            Some(k) => store::remove_by_key(&self.table, &k),
            None => store::remove_by_bytes(&self.table, buf),
        };
        if removed {
            Ok(())
        } else {
            Err(EngineError::EndOfFile)
        }
    }
}

/// `Ok(())` when the store reported a row was replaced, `EndOfFile`
/// otherwise — so a missed lookup surfaces as MySQL's documented "no
/// row matched" sentinel rather than a silent success.
fn finish_replace(replaced: bool) -> EngineResult {
    if replaced {
        Ok(())
    } else {
        Err(EngineError::EndOfFile)
    }
}
