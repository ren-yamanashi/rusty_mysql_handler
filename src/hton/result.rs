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

//! Shared translation from [`EngineResult`] to the `bool` (`true = error`)
//! convention used by handlerton callbacks that return a status flag rather
//! than an `int`. The explicit `match` form is preferred over `.is_err()` per
//! the project's "match by default at call sites" rule.
//!
//! [`EngineResult`]: crate::engine::EngineResult

use crate::engine::EngineResult;

pub(super) fn result_to_error(r: EngineResult) -> bool {
    match r {
        Ok(()) => false,
        Err(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::EngineError;

    #[test]
    fn ok_maps_to_no_error() {
        assert!(!result_to_error(Ok(())));
    }

    #[test]
    fn err_maps_to_error() {
        assert!(result_to_error(Err(EngineError::Internal)));
        assert!(result_to_error(Err(EngineError::Unsupported)));
    }
}
