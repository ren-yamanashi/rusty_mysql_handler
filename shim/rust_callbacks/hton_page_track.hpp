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

#ifndef SHIM_RUST_CALLBACKS_HTON_PAGE_TRACK_HPP
#define SHIM_RUST_CALLBACKS_HTON_PAGE_TRACK_HPP

#include <cstddef>
#include <cstdint>

// Page-tracking sub-callbacks. Wired only under
// HtonCapabilities::PAGE_TRACKING. Page_Track_Callback / context cross
// opaque (the engine that fetches page IDs will need a richer reverse
// callback). `get_status` returns no output through this layer today.
extern "C" {
int32_t rust__hton__page_track_start(uint64_t *start_id);
int32_t rust__hton__page_track_stop(uint64_t *stop_id);
int32_t rust__hton__page_track_purge(uint64_t *purge_id);
int32_t rust__hton__page_track_get_page_ids(uint64_t *start_id, uint64_t *stop_id,
                                            uint8_t *buffer, size_t buffer_len);
int32_t rust__hton__page_track_get_num_page_ids(uint64_t *start_id,
                                                uint64_t *stop_id,
                                                uint64_t *num_pages);
void rust__hton__page_track_get_status();
}

#endif
