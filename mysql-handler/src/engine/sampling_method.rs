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

//! Sampling algorithm selector, mirroring MySQL's `enum_sampling_method`.

/// Sampling algorithm requested for [`StorageEngine::sample_init`], mirroring
/// MySQL's `enum_sampling_method`.
///
/// [`StorageEngine::sample_init`]: crate::engine::StorageEngine::sample_init
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SamplingMethod {
    /// `SYSTEM` sampling: page/block-level, the only method MySQL supports today
    System,
    /// No sampling method selected
    None,
}

impl SamplingMethod {
    /// Map the raw `enum_sampling_method` integer supplied at the FFI boundary
    /// to a variant. Any value other than `SYSTEM` (`0`) becomes
    /// [`SamplingMethod::None`] so the engine never observes an undefined value.
    pub(crate) fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::System,
            _ => Self::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_maps_from_zero() {
        assert_eq!(SamplingMethod::from_raw(0), SamplingMethod::System);
    }

    #[test]
    fn non_system_codes_become_none() {
        assert_eq!(SamplingMethod::from_raw(1), SamplingMethod::None);
        assert_eq!(SamplingMethod::from_raw(-1), SamplingMethod::None);
        assert_eq!(SamplingMethod::from_raw(42), SamplingMethod::None);
    }
}
