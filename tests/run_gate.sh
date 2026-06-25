#!/data/data/com.termux/files/usr/bin/bash

set -e

BIN="./target/debug/tscp-cli"

echo "=== TSCP v0.1 Gate Test ==="

$BIN verify --envelope fixtures/test.json
$BIN verify --envelope fixtures/bad-schema.json
$BIN verify --envelope fixtures/unknown-engine.json
$BIN verify --envelope fixtures/bad-claim.json

echo "=== Gate execution complete ==="
