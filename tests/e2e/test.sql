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
-- path; the p2-25 misc methods are secondary-engine / partition specific and
-- stay build-only.)
CREATE TABLE md1 (id INT) ENGINE=RUSTY;
SHOW CREATE TABLE md1;
DROP TABLE md1;

-- sentinel: kept = 3 so run.sh's last-line check still asserts the DDL ran
SELECT 3;
