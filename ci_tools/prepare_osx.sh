#!/bin/sh
set -ex
VERSION=9.5.0
PG_DIR=/usr/local/var/postgres
HBA_CONF=$PG_DIR/pg_hba.conf
CONF=$PG_DIR/postgresql.conf
brew remove postgresql
brew update
brew install postgresql
brew cleanup postgresql
pg_ctl -D /usr/local/var/postgres -l /usr/local/var/postgres/server.log start
sleep 2
pg_ctl -D /usr/local/var/postgres stop -s -m fast || true
make clean all install
echo "local    replication     all trust" >> $HBA_CONF
pg_ctl -D /usr/local/var/postgres -l /usr/local/var/postgres/server.log start
sleep 2
pg_ctl -D /usr/local/var/postgres stop -s -m fast || true
make clean all install
echo "local    replication     all trust" >> $HBA_CONF
echo "host     replication      all ::1/128  trust" >> $HBA_CONF
echo "max_wal_senders = 1" >> $CONF
echo "wal_level=logical" >> $CONF
echo "max_replication_slots=1" >> $CONF
pg_ctl -D /usr/local/var/postgres -l /usr/local/var/postgres/server.log start
sleep 2
cat  /usr/local/var/postgres/server.log
ps ax|grep post
