#!/usr/bin/env bash
set -e
echo "[pascal] compiling legacy.pas"
fpc -O2 -Xs legacy.pas
echo "[pascal] running legacy CSV generator and importer"
./legacy