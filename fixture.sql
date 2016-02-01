SELECT * FROM pg_drop_replication_slot('jsoncdc');
SELECT * FROM pg_create_logical_replication_slot('jsoncdc', 'jsoncdc');
BEGIN;
CREATE SCHEMA IF NOT EXISTS jsoncdc;
CREATE TABLE IF NOT EXISTS jsoncdc.test (
  i   integer NOT NULL,
  t   timestamptz NOT NULL DEFAULT NOW(),
  h   hstore NOT NULL DEFAULT ''
);
INSERT INTO jsoncdc.test (i, h) VALUES (7, 'a => 7');
INSERT INTO jsoncdc.test (i, h) VALUES (9, 'a => 9');
END;
SELECT * FROM pg_logical_slot_get_changes('jsoncdc', NULL, NULL);
SELECT * FROM pg_drop_replication_slot('jsoncdc');
