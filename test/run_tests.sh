#!/bin/bash

set -ex

TEST_RESULTS_FILENAME=/tmp/results.txt
pg_recvlogical -d postgres --slot test_slot --create-slot -P jsoncdc
pg_recvlogical -d postgres --slot test_slot --start -f ${TEST_RESULTS_FILENAME} &
PWD=`pwd`
psql -d postgres -f $PWD/test/test_fixture.sql
sleep 1
sync
cat $TEST_RESULTS_FILENAME

echo "Test For Begin"
cat $TEST_RESULTS_FILENAME | grep BEGIN\"

echo "Test For Insert"
cat $TEST_RESULTS_FILENAME | grep INSERT\"

echo "Test For Commit"
cat $TEST_RESULTS_FILENAME | grep COMMIT\"
