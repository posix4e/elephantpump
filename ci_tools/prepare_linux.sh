#!/bin/sh
set -ex
VERSION=9.4
sudo pg_ctlcluster $VERSION main stop
make clean all 
sudo /bin/mkdir -p /usr/share/postgresql/${VERSION}/extension
sudo /bin/mkdir -p /usr/share/postgresql/${VERSION}/extension
sudo /bin/mkdir -p /usr/lib/postgresql/${VERSION}/lib
sudo /bin/mkdir -p /usr/share/doc/postgresql-doc-${VERSION}/extension
sudo /usr/bin/install -c -m 644 jsoncdc.control /usr/share/postgresql/${VERSION}/extension/
sudo /usr/bin/install -c -m 644 sql/jsoncdc--0.1.0.sql /usr/share/postgresql/${VERSION}/extension/
sudo /usr/bin/install -c -m 644 doc/jsoncdc.md /usr/share/doc/postgresql-doc-${VERSION}/extension/
sudo /usr/bin/install -c -m 755  jsoncdc.so /usr/lib/postgresql/${VERSION}/lib/
PG_DIR=/etc/postgresql/${VERSION}/main
HBA_CONF=$PG_DIR/pg_hba.conf
CONF=$PG_DIR/postgresql.conf

echo "local    replication     all trust" | sudo tee -a $HBA_CONF
echo "host     replication     all ::1/128 trust" | sudo tee -a $HBA_CONF
echo "max_wal_senders = 1" | sudo tee -a $CONF
echo "wal_level=logical" | sudo tee -a $CONF
echo "max_replication_slots=1" | sudo tee -a $CONF

sudo pg_ctlcluster $VERSION main start
ps axu
