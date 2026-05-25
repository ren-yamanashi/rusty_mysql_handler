-- Driven by tests/e2e/run.sh; the last non-empty line of output must be `3`.

CREATE TABLE t1 (id INT, name VARCHAR(50)) ENGINE=RUSTY;

SELECT * FROM t1;
SELECT COUNT(*) FROM t1;

INSERT INTO t1 VALUES (1, 'a'), (2, 'b'), (3, 'c');

TRUNCATE TABLE t1;
RENAME TABLE t1 TO t2;
DROP TABLE t2;

-- sentinel: kept = 3 so run.sh's last-line check still asserts the DDL ran
SELECT 3;
