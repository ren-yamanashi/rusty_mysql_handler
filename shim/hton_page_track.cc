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

// Page-tracking sub-callbacks (handler.h Page_track_t). Wired only under
// HtonCapabilities::PAGE_TRACKING — the wire function assigns
// hton->page_track as a unit. The Page_Track_Callback / context (used by
// get_page_ids to receive page IDs from the engine) is opaque to Rust
// today, so the engine sees only the (start, stop) range and the
// destination buffer. `get_status` returns a `std::vector` by value
// upstream; the FFI side invokes the trait but cannot surface the vector
// content yet.

#include "binding.hpp"
#include "rust_callbacks.hpp"
#include "sql/handler.h"

namespace {
int rusty_hton_page_track_start(uint64_t *start_id) {
  return rust__hton__page_track_start(start_id);
}

int rusty_hton_page_track_stop(uint64_t *stop_id) {
  return rust__hton__page_track_stop(stop_id);
}

int rusty_hton_page_track_purge(uint64_t *purge_id) {
  return rust__hton__page_track_purge(purge_id);
}

int rusty_hton_page_track_get_page_ids(Page_Track_Callback /* cbk_func */,
                                       void * /* cbk_ctx */,
                                       uint64_t *start_id, uint64_t *stop_id,
                                       unsigned char *buffer,
                                       size_t buffer_len) {
  return rust__hton__page_track_get_page_ids(start_id, stop_id, buffer,
                                             buffer_len);
}

int rusty_hton_page_track_get_num_page_ids(uint64_t *start_id,
                                           uint64_t *stop_id,
                                           uint64_t *num_pages) {
  return rust__hton__page_track_get_num_page_ids(start_id, stop_id, num_pages);
}

void rusty_hton_page_track_get_status(
    std::vector<std::pair<uint64_t, bool>> & /* status */) {
  // The Rust side cannot synthesise an std::vector through the opaque
  // pass-through; the trait method is invoked but no output is published.
  rust__hton__page_track_get_status();
}
}  // namespace

void rusty_hton_wire_page_track(handlerton *hton) {
  hton->page_track.start = rusty_hton_page_track_start;
  hton->page_track.stop = rusty_hton_page_track_stop;
  hton->page_track.purge = rusty_hton_page_track_purge;
  hton->page_track.get_page_ids = rusty_hton_page_track_get_page_ids;
  hton->page_track.get_num_page_ids = rusty_hton_page_track_get_num_page_ids;
  hton->page_track.get_status = rusty_hton_page_track_get_status;
}
