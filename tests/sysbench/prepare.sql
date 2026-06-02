-- Copyright (C) 2026 ren-yamanashi
--
-- Plugin install and engine selection helpers. Invoked once per harness
-- run from lib/mysqld.sh after mysqld reaches steady state.

INSTALL PLUGIN rusty SONAME 'ha_rusty.so';
CREATE DATABASE IF NOT EXISTS sbtest;
