# jsoncdc

A rust program which takes a logical decoding stream from postgresql and 
outputs it in a pluggable and safe way. Not to be confused with
[The best rust postgresql library](https://github.com/sfackler/rust-postgres).
Our library Requires rust stable 1.1 or greater.  You can bug 
posix4e(or posix4e_) on freenode if you want to get involved. Tasks to work on 
should be available on:
[![HuBoard badge](http://img.shields.io/badge/Hu-Board-7965cc.svg)](https://huboard.com/posix4e/jsoncdc)
[![Linux Status](https://travis-ci.org/posix4e/jsoncdc.svg?branch=master)](https://travis-ci.org/posix4e/jsoncdc)

## To build

You should checkout the .travis.yml for the dependencies on ubuntu to build.
Make sure to generate the postgresql bindings with the generate_bindings.sh.
You will also need python

## Formats

- [ ] Json output
- [ ] Protobufs output
- [ ] Avro output

## Destinations

- [ ] File
- [ ] Http
- [ ] kafka
- [ ] kinesis

## Features

- [ ] Monitored by rust metrics
