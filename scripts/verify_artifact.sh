#!/bin/bash
set -e

echo "TSCP Artifact Verification"

sha256sum -c events.cbor.sha256

echo "Replaying artifact..."

cargo run -p tscp-cli -- replay events.cbor

echo "Verification complete"
