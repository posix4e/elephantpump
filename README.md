JSONCDC
=======

[![Join the chat at https://gitter.im/posix4e/jsoncdc](https://badges.gitter.im/posix4e/jsoncdc.svg)](https://gitter.im/posix4e/jsoncdc?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
[![HuBoard badge](http://img.shields.io/badge/Hu-Board-7965cc.svg)](https://huboard.com/posix4e/jsoncdc)
[![Linux Status](https://travis-ci.org/posix4e/jsoncdc.svg?branch=master)](https://travis-ci.org/posix4e/jsoncdc)
[![Chat on Freenode](https://img.shields.io/badge/freenode-%23jsoncdc-brightgreen.svg)](irc://chat.freenode.net/jsoncdc)

JSONCDC provides change data capture for Postgres, translating the Postgres
write ahead log to JSON.

It is written in Rust and, being short, is a good skeleton project for other
would be plugin authors who'd like to use Rust to write Postgres extensions.

Our library Requires rust stable 1.1 or greater.



Copyright and License
---------------------

Copyright (c) 2016 Alex Newman, Jason Dusek

JSONCDC is available under multiple licenses:

* the same license as Postgres itself (`licenses/postgres`),

* the Apache 2.0 license (`licenses/apache`).


Status
------

JSONCDC is presently installable with `pgxn`, from the unstable channel:
`pgxn install jsoncdc --unstable`.


Usage
-----

A basic demo:

    SELECT * FROM pg_create_logical_replication_slot('jsoncdc', 'jsoncdc');
    --- Wait for some transactions, and then:
    SELECT * FROM pg_logical_slot_get_changes('jsoncdc', NULL, NULL);

The output format of `jsoncdc` is very regular, consisting of `begin`,
`table`, `insert`, `update`, `delete` and [`message`](https://www.postgresql.org/message-id/flat/56D36A2C.3070807%402ndquadrant.com#56D36A2C.3070807@2ndquadrant.com) clauses as JSON objects, one per line:

    { "begin": <xid> }
    { "schema": <column names and type>, "table": <name of table> }
    ...inserts, updates and deletes for this table...
    { "schema": <column names and type>, "table": <name of next table> }
    ...inserts, updates and deletes for next table...
    { "message": <message>, "prefix": <prefix>, "transactional": <true|false> }
    ... sent using pg_emit_logical
    { "commit": <xid>, "t": <timestamp with timezone> }

With `pg_recvlogical` and a little shell, you can leverage this very regular
formatting to get each transaction batched into a separate file:

    pg_recvlogical -S jsoncdc -d postgres:/// --start -f - |
    while read -r line
    do
      case "$line" in
        '{ "begin": '*)                # Close and reopen FD 9 for each new XID
          fields=( $line )
          xid="${fields[2]}"
          exec 9>&-
          exec 9> "txn-${xid}.json" ;;
      esac
      printf '%s\n' "$line" >&9       # Use printf because echo is non-portable
    done


Formats
-------

- [x] JSON output
