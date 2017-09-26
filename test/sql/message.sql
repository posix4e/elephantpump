\set ECHO none
SET client_min_messages TO error;
SET synchronous_commit = on;
\t on
\x off
SELECT 'created logical replication slot'
  FROM pg_create_logical_replication_slot('jsoncdc', 'jsoncdc');

BEGIN;

CREATE SCHEMA IF NOT EXISTS jsoncdc;
SET LOCAL search_path TO jsoncdc, public;

CREATE VIEW changedata AS
SELECT data FROM pg_logical_slot_get_changes('jsoncdc', NULL, NULL);

END;

BEGIN;
  SELECT 'visible'
	  FROM pg_logical_emit_message(false, 'sent first', '#1');
  SELECT 'invisible'
    FROM pg_logical_emit_message(true, 'sent second', 'invisible');
ROLLBACK;

BEGIN;
  SELECT 'non-transactional'
    FROM pg_logical_emit_message(false, 'sent third', '#2');
  SELECT 'transactional'
    FROM pg_logical_emit_message(true, 'sent fourth', '#4');
  SELECT 'non-transactional'
    FROM pg_logical_emit_message(false, 'sent fifth', '#3');
  SELECT 'json-recognization'
    FROM pg_logical_emit_message(true, 'sent json',
                                 '{"a": 1, "b": [2, 2]}'::jsonb::text);
  SELECT 'binary-encoding'
    FROM pg_logical_emit_message(true, 'sent binary', '\x00010203'::bytea);
COMMIT;

--- Displays only generated JSON, no replication slot metadata, and omits
--- transaction IDs (which would confuse the testing framework because they
--- vary from run to run).
SELECT * FROM jsoncdc.changedata
 WHERE NOT (data LIKE '{ "begin": %' OR data LIKE '{ "commit": %');

SELECT 'deleted logical replication slot'
  FROM pg_drop_replication_slot('jsoncdc');

DROP SCHEMA jsoncdc CASCADE;
