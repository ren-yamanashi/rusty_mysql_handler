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

//! `ha_stat_type` from `sql/handler.h`.

/// Which subset of engine status MySQL wants for `SHOW ENGINE <name> STATUS`.
///
/// Mirrors `enum ha_stat_type` in `mysql-server/sql/handler.h`. Used as the
/// argument to [`Handlerton::show_status`](crate::hton::Handlerton::show_status).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum HaStatType {
    /// `HA_ENGINE_STATUS`: `SHOW ENGINE <name> STATUS` — the engine's general
    /// status block.
    Status,
    /// `HA_ENGINE_LOGS`: `SHOW ENGINE <name> LOGS` — log-file inventory.
    Logs,
    /// `HA_ENGINE_MUTEX`: `SHOW ENGINE <name> MUTEX` — mutex / lock statistics.
    Mutex,
}

impl HaStatType {
    /// Decode the C `enum ha_stat_type` value. Unknown values map to
    /// [`HaStatType::Status`] so the engine still observes a defined variant.
    #[must_use]
    pub const fn from_raw(value: u32) -> Self {
        match value {
            1 => Self::Logs,
            2 => Self::Mutex,
            _ => Self::Status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_raw_maps_each_known_value() {
        assert_eq!(HaStatType::from_raw(0), HaStatType::Status);
        assert_eq!(HaStatType::from_raw(1), HaStatType::Logs);
        assert_eq!(HaStatType::from_raw(2), HaStatType::Mutex);
    }

    #[test]
    fn from_raw_unknown_value_falls_back_to_status() {
        assert_eq!(HaStatType::from_raw(5), HaStatType::Status);
        assert_eq!(HaStatType::from_raw(u32::MAX), HaStatType::Status);
    }
}
