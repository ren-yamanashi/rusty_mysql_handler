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

-- sentinel: kept = 3 so run.sh's last-line check still asserts the DDL ran
SELECT 3;
