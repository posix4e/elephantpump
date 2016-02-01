CREATE EXTENSION HSTORE;
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
