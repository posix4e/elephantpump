jsoncdc
=======

Synopsis
--------

  Translates Postgres WAL to JSON with Logical Decoding.

Description
-----------

  Inspired by DDP and RethinkDB's changefeeds, the `jsoncdc` extension
  provides a schema aware Logical Decoding plugin that translates Postgres WAL
  to JSON.

  The JSON for each transaction includes the transaction ID, transaction
  timestamp, a `{ "table": ... }` entry describing the schema of each affected
  table, and following each table entry, an entry for each `INSERT`, `UPDATE`
  and `DELETE` on that table.

Usage
-----

    SELECT * FROM pg_create_logical_replication_slot('jsoncdc', 'jsoncdc');
    --- Wait for some transactions, and then:
    SELECT * FROM pg_logical_slot_get_changes('jsoncdc', NULL, NULL);

Support
-------

  https://github.com/posix4e/jsoncdc
  https://github.com/posix4e/jsoncdc/issues

Author
------

  * [Alex Newman](https://github.com/posix4e)
  * [Jason Dusek](https://github.com/solidsnack)

Copyright and License
---------------------

Copyright (c) 2016 Alex Newman, Jason Dusek

