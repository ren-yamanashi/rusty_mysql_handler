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

//! `ha_notification_type` and `SelectExecutedIn` from `sql/handler.h`.

/// Whether the notification fires before (pre) or after (post) the event.
///
/// Mirrors `enum ha_notification_type` in `mysql-server/sql/handler.h`. The
/// pre-event variant is the engine's chance to veto by returning an error from
/// the notification trait method; post-event is informational.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum HaNotificationType {
    /// `HA_NOTIFY_PRE_EVENT`: the engine is asked before the event happens.
    PreEvent,
    /// `HA_NOTIFY_POST_EVENT`: the event already occurred.
    PostEvent,
}

impl HaNotificationType {
    /// Decode the C `enum ha_notification_type` value. Unknown values fall back
    /// to [`HaNotificationType::PostEvent`] so the engine still observes a
    /// defined variant.
    #[must_use]
    pub const fn from_raw(value: i32) -> Self {
        match value {
            0 => Self::PreEvent,
            _ => Self::PostEvent,
        }
    }
}

/// Which engine MySQL executed a `SELECT` on, exposed to the post-select
/// notification.
///
/// Mirrors `enum class SelectExecutedIn : bool` in `mysql-server/sql/handler.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SelectExecutedIn {
    /// `kPrimaryEngine`: the primary (this) engine ran the query.
    PrimaryEngine,
    /// `kSecondaryEngine`: a secondary engine ran the query.
    SecondaryEngine,
}

impl SelectExecutedIn {
    /// Decode the C `enum class SelectExecutedIn : bool` value (raw `0` =
    /// primary, `1` = secondary).
    #[must_use]
    pub const fn from_raw(value: bool) -> Self {
        if value {
            Self::SecondaryEngine
        } else {
            Self::PrimaryEngine
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_type_from_raw() {
        assert_eq!(
            HaNotificationType::from_raw(0),
            HaNotificationType::PreEvent
        );
        assert_eq!(
            HaNotificationType::from_raw(1),
            HaNotificationType::PostEvent
        );
        assert_eq!(
            HaNotificationType::from_raw(7),
            HaNotificationType::PostEvent
        );
    }

    #[test]
    fn select_executed_in_from_raw() {
        assert_eq!(
            SelectExecutedIn::from_raw(false),
            SelectExecutedIn::PrimaryEngine
        );
        assert_eq!(
            SelectExecutedIn::from_raw(true),
            SelectExecutedIn::SecondaryEngine
        );
    }
}
