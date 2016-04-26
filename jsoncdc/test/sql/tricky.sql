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

CREATE TABLE IF NOT EXISTS tab1 (
  i   integer PRIMARY KEY,
  h   hstore NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS tab2 (
  i   integer PRIMARY KEY,
  j   integer NOT NULL DEFAULT 0
);

INSERT INTO tab1 (i) VALUES (1);
INSERT INTO tab2 (i) VALUES (1);

INSERT INTO tab1 (i) VALUES (2);
INSERT INTO tab2 (i) VALUES (2);

INSERT INTO tab1 (i) VALUES (3);
INSERT INTO tab2 (i) VALUES (3);

INSERT INTO tab1 (i) VALUES (4);
INSERT INTO tab2 (i) VALUES (4);

--- Displays only generated JSON, no replication slot metadata, and omits
--- transaction IDs (which would confuse the testing framework because they
--- vary from run to run).
CREATE VIEW changedata AS
SELECT data FROM pg_logical_slot_get_changes('jsoncdc', NULL, NULL)
 WHERE NOT (data LIKE '{ "begin": %' OR data LIKE '{ "commit": %');

END;

BEGIN;
SET LOCAL search_path TO jsoncdc, public;
UPDATE tab1 SET h = hstore('i', i::text)||hstore('2i', (2*i)::text);
DELETE FROM tab1 WHERE i % 2 = 1;
DELETE FROM tab2 WHERE i % 2 = 1;
END;

SELECT * FROM jsoncdc.changedata;

SELECT 'deleted logical replication slot'
  FROM pg_drop_replication_slot('jsoncdc');

DROP SCHEMA jsoncdc CASCADE;
