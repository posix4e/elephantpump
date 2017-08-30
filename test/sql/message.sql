\set ECHO none
SET client_min_messages TO error;
SET synchronous_commit = on;
\t on
\x off
SELECT 'created logical replication slot'
  FROM pg_create_logical_replication_slot('jsoncdc', 'jsoncdc');

CREATE SCHEMA IF NOT EXISTS jsoncdc;
SET LOCAL search_path TO jsoncdc, public;

BEGIN;
	SELECT 'visible'
	  FROM pg_logical_emit_message(false, 'sent first', '#1');
  	SELECT 'invisible'
	  FROM pg_logical_emit_message(true, 'sent second', 'existence is pain');
ROLLBACK;

BEGIN;
	SELECT 'non-transactional'
	  FROM pg_logical_emit_message(false, 'sent third', '#2');
	SELECT 'transactional'
	  FROM pg_logical_emit_message(true, 'sent fourth', '#4');
  	SELECT 'non-transactional'
	  FROM pg_logical_emit_message(false, 'sent last', '#3');
COMMIT;

SELECT *
  FROM (SELECT data FROM pg_logical_slot_get_changes('jsoncdc', NULL, NULL)) t
 WHERE data
  LIKE '{ "message": %';

SELECT 'deleted logical replication slot'
  FROM pg_drop_replication_slot('jsoncdc');

DROP SCHEMA jsoncdc CASCADE;
