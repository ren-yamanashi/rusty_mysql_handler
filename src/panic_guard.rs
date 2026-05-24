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

//! FFI boundary panic safety. Every `extern "C"` callback exposed to the C++
//! shim must funnel its body through one of these helpers so that a Rust panic
//! cannot unwind across the language boundary and abort the MySQL server.

use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::engine::EngineError;

const PANIC_LOG_MSG: &str = "ffi boundary caught panic from storage engine";

/// Panic-safe entry point for every `extern "C"` callback. Zero-sized; the
/// methods are associated functions grouped by responsibility.
#[derive(Debug)]
pub struct FfiBoundary;

impl FfiBoundary {
    /// Run `f` inside `catch_unwind` and project the outcome into a MySQL
    /// `HA_ERR_*` integer:
    ///
    /// - `Ok(Ok(()))` → `0`
    /// - `Ok(Err(e))` → `e.to_mysql_errno()`
    /// - `Err(_)` (panic) → `HA_ERR_INTERNAL_ERROR`
    pub fn run<F>(f: F) -> i32
    where
        F: FnOnce() -> Result<(), EngineError>,
    {
        match catch_unwind(AssertUnwindSafe(f)) {
            Ok(Ok(())) => 0,
            Ok(Err(e)) => e.to_mysql_errno(),
            Err(_) => {
                tracing::error!("{PANIC_LOG_MSG}");
                EngineError::Internal.to_mysql_errno()
            }
        }
    }

    /// Variant for callbacks whose C++ signature returns void. Panics are
    /// swallowed so the server stays alive; errors cannot be reported back
    /// to MySQL through a void return.
    pub fn run_void<F>(f: F)
    where
        F: FnOnce(),
    {
        if let Ok(()) = catch_unwind(AssertUnwindSafe(f)) {
        } else {
            tracing::error!("{PANIC_LOG_MSG}");
        }
    }

    /// Variant for callbacks that return a non-`Result` value (pointer,
    /// integer flag bitfield, etc.). Returns `default` on panic so the caller
    /// always sees a well-defined value rather than an unwound stack.
    pub fn run_default<T, F>(default: T, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        if let Ok(v) = catch_unwind(AssertUnwindSafe(f)) {
            v
        } else {
            tracing::error!("{PANIC_LOG_MSG}");
            default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sys;

    #[test]
    fn ok_closure_returns_zero() {
        assert_eq!(FfiBoundary::run(|| Ok(())), 0);
    }

    #[test]
    fn err_closure_returns_mapped_errno() {
        assert_eq!(
            FfiBoundary::run(|| Err(EngineError::EndOfFile)),
            sys::HA_ERR_END_OF_FILE
        );
    }

    #[test]
    fn panicking_closure_returns_internal_error() {
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let result = FfiBoundary::run(|| panic!("intentional panic for test"));
        std::panic::set_hook(prev_hook);
        assert_eq!(result, sys::HA_ERR_INTERNAL_ERROR);
    }

    #[test]
    fn panicking_void_closure_does_not_abort() {
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        FfiBoundary::run_void(|| panic!("intentional panic for test"));
        std::panic::set_hook(prev_hook);
    }
}
