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
project(ha_rusty LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

if(NOT DEFINED MYSQL_SOURCE_DIR)
  message(FATAL_ERROR "Set -DMYSQL_SOURCE_DIR=<path to mysql-server source>")
endif()
if(NOT DEFINED MYSQL_BUILD_DIR)
  message(FATAL_ERROR "Set -DMYSQL_BUILD_DIR=<path to mysql cmake build dir>")
endif()

find_package(OpenSSL REQUIRED)

add_definitions(-DMYSQL_DYNAMIC_PLUGIN -DMYSQL_SERVER -DHAVE_CONFIG_H)

include_directories(
  ${MYSQL_BUILD_DIR}/include
  ${MYSQL_SOURCE_DIR}/include
  ${MYSQL_SOURCE_DIR}/sql
  ${MYSQL_SOURCE_DIR}
  ${MYSQL_SOURCE_DIR}/extra/rapidjson/include
  ${OPENSSL_INCLUDE_DIR}
)

add_library(ha_rusty MODULE binding.cc plugin.cc)
set_target_properties(ha_rusty PROPERTIES PREFIX "" SUFFIX ".so")

if(APPLE)
  target_link_options(ha_rusty PRIVATE LINKER:-undefined,dynamic_lookup)
endif()

option(ENABLE_RUST "Link Rust engine staticlib" OFF)
if(ENABLE_RUST)
  if(NOT DEFINED RUST_LIB_DIR)
    message(FATAL_ERROR "Set -DRUST_LIB_DIR=<path to Rust target/release>")
  endif()
  target_link_libraries(ha_rusty PRIVATE ${RUST_LIB_DIR}/libengine.a)
endif()
