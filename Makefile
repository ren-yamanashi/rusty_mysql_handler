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

.PHONY: setup rust-build clean lint fmt check test test_e2e help

MYSQL_SOURCE_DIR ?= $(CURDIR)/mysql-server
MYSQL_BUILD_DIR  ?= $(CURDIR)/build/mysql

setup: ## Initialize submodules, hooks, and generate MySQL headers
	@git submodule update --init --depth 1
	@git config core.hooksPath .githooks
	@$(MAKE) --no-print-directory mysql-configure

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2}'

# WITHOUT_SERVER=ON: skip building the mysqld binary itself; we only need the
# generated public headers (my_config.h + handler.h chain) for bindgen and the
# C++ shim. Cuts configure / build time substantially.
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

test_e2e: ## E2E test via Docker (mysql:8.4 + plugin baked into image)
	@bash tests/e2e/run.sh

clean: ## Remove cargo build artifacts (keeps build/mysql to avoid re-running mysql-configure)
	@cargo clean
