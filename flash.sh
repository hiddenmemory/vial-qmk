#!/bin/sh

cd rust/qmk-rs
cargo build
make build
cd ../..

qmk flash -kb mechboards/lily58/r2g -km vial
