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

.PHONY: all setup rust-build shim-build install clean lint fmt check test help

MYSQL_SOURCE_DIR ?= $(CURDIR)/mysql-server
MYSQL_BUILD_DIR  ?= $(CURDIR)/build/mysql
RUST_LIB_DIR     ?= $(CURDIR)/target/release
BUILD_DIR        ?= $(CURDIR)/build
ENABLE_RUST      ?= OFF
PLUGIN_DIR       ?= $(shell mysql -u root -N -e "SHOW VARIABLES LIKE 'plugin_dir'" 2>/dev/null | awk '{print $$2}')

all: rust-build shim-build

setup: ## Initialize submodules, hooks, and generate MySQL headers
	@git submodule update --init --depth 1
	@git config core.hooksPath .githooks
	@$(MAKE) --no-print-directory mysql-configure

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2}'

$(MYSQL_BUILD_DIR)/include/my_config.h:
	@mkdir -p $(MYSQL_BUILD_DIR)
	cd $(MYSQL_BUILD_DIR) && cmake $(MYSQL_SOURCE_DIR) \
		-DDOWNLOAD_BOOST=1 \
		-DWITH_BOOST=$(MYSQL_SOURCE_DIR)/extra/boost \
		-DWITHOUT_SERVER=ON \
		-DCMAKE_BUILD_TYPE=Release

mysql-configure: $(MYSQL_BUILD_DIR)/include/my_config.h ## Generate MySQL build headers

rust-build: ## Build all Rust crates (release)
	@cargo build --release

shim-build: mysql-configure ## Build ha_rusty.so
	@mkdir -p $(BUILD_DIR)
	@cd $(BUILD_DIR) && cmake $(CURDIR)/shim \
		-DMYSQL_SOURCE_DIR=$(MYSQL_SOURCE_DIR) \
		-DMYSQL_BUILD_DIR=$(MYSQL_BUILD_DIR) \
		-DENABLE_RUST=$(ENABLE_RUST) \
		-DRUST_LIB_DIR=$(RUST_LIB_DIR)
	@$(MAKE) --no-print-directory -C $(BUILD_DIR)

install: ## Copy ha_rusty.so to MySQL plugin_dir
	@test -n "$(PLUGIN_DIR)" || { echo "ERROR: Set PLUGIN_DIR or start MySQL."; exit 1; }
	@cp $(BUILD_DIR)/ha_rusty.so "$(PLUGIN_DIR)/"

lint: ## Run clippy
	@cargo clippy --workspace -- -D warnings

fmt: ## Format Rust code
	@cargo fmt --all

check: ## Run check + clippy + fmt check + license check
	@cargo check --workspace
	@cargo clippy --workspace -- -D warnings
	@cargo fmt --all --check
	@bash scripts/check-license.sh

test: ## Run tests
	@cargo test --workspace

clean: ## Remove build artifacts
	@cargo clean
	@rm -rf $(BUILD_DIR)
