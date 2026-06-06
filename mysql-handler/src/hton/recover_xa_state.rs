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

//! `enum_ha_recover_xa_state` from `sql/handler.h`.

/// State of an externally coordinated XA transaction reported by the
/// engine to the transaction coordinator during recovery.
///
/// Mirrors `enum class enum_ha_recover_xa_state : int` in
/// `mysql-server/sql/handler.h`. The `NOT_FOUND = -1` sentinel exists
/// upstream as a lookup result, not as something an engine reports, so
/// it is not represented here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RecoverXaState {
    /// `PREPARED_IN_SE = 0`
    PreparedInSe,
    /// `PREPARED_IN_TC = 1`
    PreparedInTc,
    /// `COMMITTED_WITH_ONEPHASE = 2`
    CommittedWithOnephase,
    /// `COMMITTED = 3`
    Committed,
    /// `ROLLEDBACK = 4`
    RolledBack,
}

impl RecoverXaState {
    /// Encode this state as the raw `enum_ha_recover_xa_state` integer the
    /// shim hands back to MySQL.
    #[must_use]
    pub const fn to_raw(self) -> i32 {
        match self {
            Self::PreparedInSe => 0,
            Self::PreparedInTc => 1,
            Self::CommittedWithOnephase => 2,
            Self::Committed => 3,
            Self::RolledBack => 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_values_match_upstream() {
        assert_eq!(RecoverXaState::PreparedInSe.to_raw(), 0);
        assert_eq!(RecoverXaState::PreparedInTc.to_raw(), 1);
        assert_eq!(RecoverXaState::CommittedWithOnephase.to_raw(), 2);
        assert_eq!(RecoverXaState::Committed.to_raw(), 3);
        assert_eq!(RecoverXaState::RolledBack.to_raw(), 4);
    }
}
