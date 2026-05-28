# Copyright (C) 2026 ren-yamanashi
#
# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License, version 2.0,
# as published by the Free Software Foundation.
#
# This program is designed to work with certain software (including
# but not limited to OpenSSL) that is licensed under separate terms,
# as designated in a particular file or component or in included license
# documentation. The authors of this program hereby grant you an additional
# permission to link the program and your derivative works with the
# separately licensed software that they have either included with
# the program or referenced in the documentation.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program; if not, see <https://www.gnu.org/licenses/>.

cmake_minimum_required(VERSION 3.20)
project(ha_rusty_shim LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

if(NOT DEFINED MYSQL_SOURCE_DIR)
  message(FATAL_ERROR "Set -DMYSQL_SOURCE_DIR=<path to mysql-server source>")
endif()
if(NOT DEFINED MYSQL_BUILD_DIR)
  message(FATAL_ERROR "Set -DMYSQL_BUILD_DIR=<path to mysql cmake build dir>")
endif()

# Required transitively by MySQL headers (sql/handler.h pulls in
# rapidjson/x509 helpers). Not used directly by the shim sources.
find_package(OpenSSL REQUIRED)

add_definitions(-DMYSQL_DYNAMIC_PLUGIN -DMYSQL_SERVER -DHAVE_CONFIG_H)

include_directories(
  ${MYSQL_BUILD_DIR}/include
  ${MYSQL_SOURCE_DIR}/include
  ${MYSQL_SOURCE_DIR}/sql
  # MySQL 8.4 moved some headers (mysql/components/*, dd/types/*) under libs/;
  # sql/handler.h transitively requires them.
  ${MYSQL_SOURCE_DIR}/libs
  ${MYSQL_SOURCE_DIR}
  ${MYSQL_SOURCE_DIR}/extra/rapidjson/include
  ${OPENSSL_INCLUDE_DIR}
)

# Built as STATIC so a downstream Rust cdylib (built by mysql-handler's
# build.rs) can pull it in. The final ha_rusty.{so,dylib} is produced
# by `cargo build --release -p engine`, not by this CMakeLists.
add_library(ha_rusty_shim STATIC binding.cc handler_bulk_load.cc
                                 handler_bulk_operations.cc handler_caps.cc
                                 handler_cost.cc handler_error_handling.cc
                                 handler_fulltext.cc handler_hints.cc
                                 handler_index_basic.cc handler_index_pushed.cc
                                 handler_index_range.cc
                                 handler_inplace_alter.cc handler_lifecycle.cc
                                 handler_limits.cc handler_locking.cc
                                 handler_mrr.cc handler_parallel_scan.cc
                                 handler_read_removal_autoinc.cc
                                 handler_records.cc handler_row_operations.cc
                                 handler_sampling.cc plugin.cc)
