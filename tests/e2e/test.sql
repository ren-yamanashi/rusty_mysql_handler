-- Driven by tests/e2e/run.sh; the last non-empty line of output must be `3`.

CREATE TABLE t1 (id INT, name VARCHAR(50)) ENGINE=RUSTY;

SELECT * FROM t1;
SELECT COUNT(*) FROM t1;

DROP TABLE t1;
