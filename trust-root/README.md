# TSCP AVX-512 SMT seal digest, mirrored

## In plain English

This repository holds a copy of a date-stamp for a separate evidence bundle. The
evidence bundle lives in another account and could be rewritten; this copy could
not be rewritten without also rewriting here, which is a second, separate action
in a second, separate place. Two locks on two doors is not the same as a witness,
but it is harder than one lock.

## What is here

- `SEAL_AGGREGATE_DIGEST.txt` and `.ots`: first aggregate, covering the five
  manifests as they stood before the NEON models were bound to their source.
- `SEAL_AGGREGATE_DIGEST_v2.txt` and `.ots`: second aggregate, covering the five
  manifests after that change. v2 supersedes v1 by addition; v1 is kept because
  it describes a real earlier state.

## The values

| record | digest over all manifests |
|---|---|
| v1 aggregate | `6fdbe7bc14502db3d1c223617106866c6ff808e7a296f392309edf7d47ca226c` |
| v2 aggregate | `d4ef6e8c643f515badd57dc6465536303e4b56838ddf4dbaad0f1ac4ef79b080` |

## Where these came from

Published from `Cartilage-Stairwells/avx512-butterfly-evidence`, at the commit
that added the NEON source binding. The recorded `recorded_utc` and
`supersedes` fields inside each file name the exact state each one describes.

## How to check a copy

Recompute the manifest digests in the evidence bundle, join them in the order
named in the aggregate file, hash that, and compare with the table above. Then
verify the `.ots` proof against the aggregate file it sits beside.

## What this does and does not establish

It establishes that these digest values existed at a particular time, recorded
outside the repository that produced them, and that a copy of the timestamp
survives independently of that repository.

It does NOT establish that the digests describe correct work, only that they are
old and unchanged since the date shown. And both places are controlled by the
same operator, so this raises the cost of a rewrite rather than making one
impossible. A genuinely independent witness would be a third party, or a public
append-only log the originator cannot edit. Neither exists yet.

## Current timestamp state, measured

v1 is Bitcoin-confirmed. Its proof carries attestations in blocks 974947 and
974953, and both merkle roots were checked against a public block explorer
independently of the proof itself, so the date is anchored rather than merely
submitted. The stamped digest is unchanged by the upgrade; upgrading only adds
attestations.

v2 is still pending on all four calendars. A pending proof is a receipt for a
submission, not a settled record.

Re-check the `.ots` files for the present state rather than trusting this
paragraph, and verify any settled block against a block explorer rather than
against the proof's own claim about itself.
At the time this mirror was written the proofs were still pending Bitcoin
confirmation. A pending proof is a receipt for a submission, not a settled
record. Check the `.ots` files for their present state rather than trusting
this paragraph.
