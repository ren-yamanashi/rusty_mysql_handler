-- Driven by tests/e2e/run.sh; the last non-empty line of output must be `3`.

CREATE TABLE t1 (id INT, name VARCHAR(50)) ENGINE=RUSTY;

SELECT * FROM t1;
SELECT COUNT(*) FROM t1;

INSERT INTO t1 VALUES (1, 'a'), (2, 'b'), (3, 'c');

-- Cost-estimation paths: EXPLAIN drives scan_time / table_scan_cost / read_cost
-- through the optimizer; COUNT(*) drives the records() exact-count path.
EXPLAIN SELECT * FROM t1;
EXPLAIN SELECT * FROM t1 WHERE id = 1;
SELECT COUNT(*) FROM t1;

-- Drives the sampling handler path (sample_init -> sample_next -> sample_end);
-- histogram building calls it unconditionally via ha_sample_*.
ANALYZE TABLE t1 UPDATE HISTOGRAM ON id WITH 10 BUCKETS;

TRUNCATE TABLE t1;
RENAME TABLE t1 TO t2;
DROP TABLE t2;

-- max_supported_keys() >= 1 lets an indexed table be created, which makes the
-- index and range-scan handler paths reachable from SQL.
-- id is NOT NULL because TrivialEngine's index_flags does not advertise
-- HA_NULL_IN_KEY (a capability bound later); a nullable indexed column is
-- rejected with ER_NULL_COLUMN_IN_INDEX otherwise.
CREATE TABLE idx1 (id INT NOT NULL, name VARCHAR(50), KEY idx_id (id)) ENGINE=RUSTY;

-- index_read_map / index_first / index_next (equality + ordered scan)
SELECT * FROM idx1 WHERE id = 1;
SELECT * FROM idx1 ORDER BY id;
-- index_last / index_prev (reverse ordered scan)
SELECT * FROM idx1 ORDER BY id DESC;
-- read_range_first / read_range_next (range scan)
SELECT * FROM idx1 WHERE id BETWEEN 1 AND 10;
-- records_in_range (optimizer row-count estimate)
EXPLAIN SELECT * FROM idx1 WHERE id > 5;

-- get_real_row_type / get_default_index_algorithm (capability queries)
SHOW CREATE TABLE idx1;
SHOW TABLE STATUS LIKE 'idx1';

DROP TABLE idx1;

-- Locking paths: external_lock(F_WRLCK) + start_stmt on LOCK, external_lock(F_UNLCK) on UNLOCK.
CREATE TABLE lk1 (id INT) ENGINE=RUSTY;
LOCK TABLES lk1 WRITE;
SELECT * FROM lk1;
UNLOCK TABLES;
DROP TABLE lk1;

-- Auto-increment path: get_auto_increment / release_auto_increment on INSERT ... VALUES ().
CREATE TABLE ai1 (id INT NOT NULL AUTO_INCREMENT PRIMARY KEY) ENGINE=RUSTY;
INSERT INTO ai1 VALUES ();
INSERT INTO ai1 VALUES ();
DROP TABLE ai1;

-- extra (HA_EXTRA_* hints) and reset run implicitly on every statement above,
-- so the hint/reset bindings are already exercised by the normal query path.
-- The SQL HANDLER interface needs engine support TrivialEngine does not provide,
-- so init_table_handle_for_HANDLER stays build-only.

-- In-place ALTER path: check_if_supported_inplace_alter (and the copy fallback
-- it selects) on ADD COLUMN.
CREATE TABLE al1 (id INT) ENGINE=RUSTY;
ALTER TABLE al1 ADD COLUMN name VARCHAR(50);
SHOW CREATE TABLE al1;
DROP TABLE al1;

-- Table-maintenance admin commands: check / analyze / optimize / repair. The
-- engine declines (base fallback) so each returns a "not implemented" note
-- rather than an error.
CREATE TABLE mt1 (id INT) ENGINE=RUSTY;
CHECK TABLE mt1;
ANALYZE TABLE mt1;
OPTIMIZE TABLE mt1;
REPAIR TABLE mt1;
DROP TABLE mt1;

-- Create-info / metadata: SHOW CREATE TABLE drives update_create_info and
-- append_create_info. (cmp_ref / set_ha_share_ref run on the open/positioning
-- path; the remaining misc methods are secondary-engine / partition specific
-- and stay build-only.)
CREATE TABLE md1 (id INT) ENGINE=RUSTY;
SHOW CREATE TABLE md1;
DROP TABLE md1;

-- Row visibility: INSERT must store the actual row bytes so a later SELECT
-- returns the inserted values (not empty rows). The previous version of
-- TrivialEngine kept only counts; after the demo bring-up it stores the raw
-- record image so SUM / specific-value lookups return real data.
CREATE TABLE rv (id INT, label VARCHAR(20)) ENGINE=RUSTY;
INSERT INTO rv VALUES (10, 'a'), (20, 'b'), (30, 'c');
SELECT @rv_sum := SUM(id), @rv_count := COUNT(*) FROM rv;
DROP TABLE rv;

-- CRUD on an indexed table: UPDATE / DELETE locate the target row through
-- MySQL's filtering, and the demo engine modifies the committed store
-- in-place. The reference engine does not implement transactional UPDATE /
-- DELETE, so these run under autocommit only.
CREATE TABLE crud (id INT NOT NULL, label VARCHAR(20), KEY idx_id (id)) ENGINE=RUSTY;
INSERT INTO crud VALUES (10, 'a'), (20, 'b'), (30, 'c');
UPDATE crud SET label = 'X' WHERE id = 20;
DELETE FROM crud WHERE id = 10;
-- Zero-match UPDATE / DELETE: must leave the row set untouched and not
-- surface a spurious EndOfFile to the client. The Rust EndOfFile from
-- update_row / delete_row is the "no row matched" signal MySQL silently
-- absorbs as `Rows matched: 0`.
UPDATE crud SET label = 'Z' WHERE id = 999;
DELETE FROM crud WHERE id = 999;
SELECT @crud_sum := SUM(id), @crud_count := COUNT(*) FROM crud;
SELECT @crud_label_20 := label FROM crud WHERE id = 20;
DROP TABLE crud;

-- Sortable storage: a 5-row table to exercise partial BETWEEN ranges
-- (the engine must yield only rows inside the bounds, since HA_READ_RANGE
-- is now advertised and the server no longer re-filters), and ORDER BY
-- ASC / DESC with LIMIT 1 to confirm `index_first` / `index_last` return
-- the actual endpoints (the server trusts HA_READ_ORDER and does not
-- sort again).
CREATE TABLE rng (id INT NOT NULL, KEY idx_id (id)) ENGINE=RUSTY;
INSERT INTO rng VALUES (1), (2), (3), (4), (5);
SELECT @rng_between_sum := SUM(id) FROM rng WHERE id BETWEEN 2 AND 3;
SELECT @rng_between_count := COUNT(*) FROM rng WHERE id BETWEEN 2 AND 3;
SELECT @rng_first := id FROM rng ORDER BY id LIMIT 1;
SELECT @rng_last := id FROM rng ORDER BY id DESC LIMIT 1;
DROP TABLE rng;

-- Non-default key offset: `pad` (INT NOT NULL, 4 bytes) sits in front of
-- `id`, so the indexed column starts at byte 5 in record[0] (1 null bits
-- byte + 4 bytes of pad). A regression that silently falls back to
-- DEFAULT_KEY_OFFSET = 1 would compare the WHERE-clause key against pad's
-- bytes and never match any row, so WHERE id = ? would return an empty
-- set and UPDATE / DELETE would be no-ops. Verify all three locate id
-- correctly.
CREATE TABLE crud_off (pad INT NOT NULL, id INT NOT NULL, label VARCHAR(20), KEY idx_id (id)) ENGINE=RUSTY;
INSERT INTO crud_off VALUES (0, 10, 'a'), (0, 20, 'b'), (0, 30, 'c');
UPDATE crud_off SET label = 'Y' WHERE id = 20;
DELETE FROM crud_off WHERE id = 10;
SELECT @crud_off_sum := SUM(id), @crud_off_count := COUNT(*) FROM crud_off;
SELECT @crud_off_label_20 := label FROM crud_off WHERE id = 20;
DROP TABLE crud_off;

-- BEGIN..UPDATE..ROLLBACK does NOT undo the change for the reference
-- engine — UPDATE mutates the committed store directly. The assertion
-- below pins this documented limitation: the post-rollback label is the
-- new value, not the pre-update one.
CREATE TABLE crud_tx (id INT NOT NULL, label VARCHAR(20), KEY idx_id (id)) ENGINE=RUSTY;
INSERT INTO crud_tx VALUES (1, 'before');
BEGIN;
UPDATE crud_tx SET label = 'after' WHERE id = 1;
ROLLBACK;
SELECT @crud_tx_label := label FROM crud_tx WHERE id = 1;
DROP TABLE crud_tx;

-- Transaction observability: a transactional handlerton registers in
-- external_lock when a statement touches a RUSTY table, so COMMIT must make its
-- insert durable to the next statement and ROLLBACK must discard its insert.
-- Capture the count after COMMIT and again after ROLLBACK so each half is
-- asserted independently (a COMMIT that drops its row and a ROLLBACK that keeps
-- its row would both net to 1 and slip past a single net check). run.sh only
-- inspects the last non-empty line, so the final sentinel is 3 only when
-- after-commit == 1 (COMMIT persisted) AND after-rollback == 1 (ROLLBACK
-- discarded).
CREATE TABLE tx1 (id INT) ENGINE=RUSTY;
BEGIN;
INSERT INTO tx1 VALUES (1);
COMMIT;
SELECT @after_commit := COUNT(*) FROM tx1;
BEGIN;
INSERT INTO tx1 VALUES (2);
ROLLBACK;
SELECT @after_rollback := COUNT(*) FROM tx1;
DROP TABLE tx1;

-- Savepoint observability: ROLLBACK TO SAVEPOINT must undo the insert done
-- after the savepoint while keeping the one before it, so the committed count
-- is 1 (not 2 if the savepoint rollback were a no-op).
CREATE TABLE sp1 (id INT) ENGINE=RUSTY;
BEGIN;
INSERT INTO sp1 VALUES (1);
SAVEPOINT s1;
INSERT INTO sp1 VALUES (2);
ROLLBACK TO SAVEPOINT s1;
COMMIT;
SELECT @after_savepoint := COUNT(*) FROM sp1;
DROP TABLE sp1;

-- RELEASE SAVEPOINT keeps the released savepoint's work (the engine's
-- savepoint_release callback only drops its snapshot), so both inserts commit
-- and the count is 2.
CREATE TABLE sp2 (id INT) ENGINE=RUSTY;
BEGIN;
INSERT INTO sp2 VALUES (1);
SAVEPOINT r1;
INSERT INTO sp2 VALUES (2);
RELEASE SAVEPOINT r1;
COMMIT;
SELECT @after_release := COUNT(*) FROM sp2;
DROP TABLE sp2;

-- Engine-level status hooks: SHOW ENGINE RUSTY STATUS drives show_status (the
-- TrivialHandlerton emits one demo row), FLUSH LOGS drives flush_logs and must
-- return success. Discovery hooks fire passively during DDL above; the load
-- itself verifies the always-wire stubs do not break the plugin.
SHOW ENGINE RUSTY STATUS;
FLUSH LOGS;

-- drop_database hook fires once per schema dropped. A RUSTY table inside the
-- schema ensures the handlerton sees the notification rather than another
-- engine swallowing it.
CREATE DATABASE rusty_drop_db_test;
CREATE TABLE rusty_drop_db_test.t1 (id INT) ENGINE=RUSTY;
DROP DATABASE rusty_drop_db_test;

-- sentinel: 3 only when COMMIT persisted (1), ROLLBACK discarded (1),
-- ROLLBACK TO SAVEPOINT undid only the post-savepoint insert (1),
-- RELEASE SAVEPOINT kept both inserts (2), INSERT made real row values
-- visible to SELECT (sum 60, count 3), UPDATE / DELETE found and
-- modified the right rows (UPDATE id=20 to 'X', DELETE id=10 → remaining
-- sum 50, count 2, label of id=20 is 'X'), the same UPDATE / DELETE /
-- SELECT path located id correctly even when it lives at a non-default
-- offset (sum 50, count 2, label of id=20 is 'Y'), zero-match UPDATE /
-- DELETE left the row set unchanged, and BEGIN..UPDATE..ROLLBACK around
-- UPDATE did not undo the change (the engine's documented
-- non-transactional UPDATE limitation — the label after ROLLBACK is
-- 'after', not 'before').
SELECT IF(
  @after_commit = 1 AND @after_rollback = 1 AND @after_savepoint = 1 AND @after_release = 2
  AND @rv_sum = 60 AND @rv_count = 3
  AND @crud_sum = 50 AND @crud_count = 2 AND @crud_label_20 = 'X'
  AND @crud_off_sum = 50 AND @crud_off_count = 2 AND @crud_off_label_20 = 'Y'
  AND @crud_tx_label = 'after'
  AND @rng_between_sum = 5 AND @rng_between_count = 2
  AND @rng_first = 1 AND @rng_last = 5,
  3, 0
) AS sentinel;
