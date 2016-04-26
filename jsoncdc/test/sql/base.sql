\set ECHO none
SET client_min_messages TO error;
\t on
\x off
SELECT 'created logical replication slot'
  FROM pg_create_logical_replication_slot('jsoncdc', 'jsoncdc');

BEGIN;

CREATE SCHEMA IF NOT EXISTS jsoncdc;
SET LOCAL search_path TO jsoncdc, public;
CREATE EXTENSION IF NOT EXISTS hstore;

CREATE TABLE IF NOT EXISTS test (
  i   integer PRIMARY KEY,
  h   hstore NOT NULL DEFAULT ''
);

INSERT INTO test (i) SELECT i FROM generate_series(1, 8) AS i;

CREATE VIEW changedata AS
SELECT data FROM pg_logical_slot_get_changes('jsoncdc', NULL, NULL);

END;

BEGIN;
SET LOCAL search_path TO jsoncdc, public;
UPDATE test SET i = i * 10 WHERE i % 3 = 0;
UPDATE test SET h = hstore('i', i::text)||hstore('2i', (2*i)::text);
DELETE FROM test WHERE i % 2 = 1;
END;

--- Displays only generated JSON, no replication slot metadata, and omits
--- transaction IDs (which would confuse the testing framework because they
--- vary from run to run).
SELECT * FROM jsoncdc.changedata
 WHERE NOT (data LIKE '{ "begin": %' OR data LIKE '{ "commit": %');

SELECT 'deleted logical replication slot'
  FROM pg_drop_replication_slot('jsoncdc');

DROP SCHEMA jsoncdc CASCADE;
