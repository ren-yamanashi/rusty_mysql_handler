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

//! `SecondaryEngineGraphSimplificationRequest` enum and its
//! `SecondaryEngineGraphSimplificationRequestParameters` struct from
//! `sql/handler.h`. Returned by [`Handlerton::secondary_engine_check_optimizer_request`].
//!
//! [`Handlerton::secondary_engine_check_optimizer_request`]: crate::hton::Handlerton::secondary_engine_check_optimizer_request

/// What the secondary engine asks the hypergraph optimizer to do next.
///
/// Mirrors `enum class SecondaryEngineGraphSimplificationRequest` in
/// `mysql-server/sql/handler.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SecondaryEngineGraphSimplificationRequest {
    /// `kContinue = 0`: continue optimization with the current hypergraph.
    Continue,
    /// `kRestart = 1`: restart hypergraph with the provided subgraph-pair count.
    Restart,
}

impl SecondaryEngineGraphSimplificationRequest {
    /// Convert to the raw C value the shim packs into the C++ struct.
    #[must_use]
    pub const fn to_raw(self) -> i32 {
        match self {
            Self::Continue => 0,
            Self::Restart => 1,
        }
    }
}

/// The pair the secondary engine returns from
/// [`Handlerton::secondary_engine_check_optimizer_request`].
///
/// Mirrors `struct SecondaryEngineGraphSimplificationRequestParameters` in
/// `mysql-server/sql/handler.h`.
///
/// [`Handlerton::secondary_engine_check_optimizer_request`]: crate::hton::Handlerton::secondary_engine_check_optimizer_request
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecondaryEngineOptimizerRequest {
    /// The transition the engine asks the optimizer to make.
    pub request: SecondaryEngineGraphSimplificationRequest,
    /// Subgraph-pair limit for the (possibly) restarted hypergraph.
    pub subgraph_pair_limit: i32,
}

impl SecondaryEngineOptimizerRequest {
    /// "Keep going" default — what the trait returns when the engine has no
    /// opinion. Matches the upstream documented default (`kContinue`, 0).
    #[must_use]
    pub const fn keep_going() -> Self {
        Self {
            request: SecondaryEngineGraphSimplificationRequest::Continue,
            subgraph_pair_limit: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simplification_request_raw_round_trip() {
        assert_eq!(
            SecondaryEngineGraphSimplificationRequest::Continue.to_raw(),
            0
        );
        assert_eq!(
            SecondaryEngineGraphSimplificationRequest::Restart.to_raw(),
            1
        );
    }

    #[test]
    fn keep_going_default() {
        let r = SecondaryEngineOptimizerRequest::keep_going();
        assert_eq!(
            r.request,
            SecondaryEngineGraphSimplificationRequest::Continue
        );
        assert_eq!(r.subgraph_pair_limit, 0);
    }
}
