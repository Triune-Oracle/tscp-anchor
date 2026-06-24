# TSCP Technical Fingerprint

## Deterministic Transition Pipeline

EventEnvelope
    |
    v
Parent State Hash Verification
    |
    v
Deterministic State Transition
    |
    v
Child State Hash
    |
    v
TransitionReceipt

## Receipt Properties

A receipt binds:

- parent_state_hash
- event_hash
- child_state_hash
- kernel_version

## Required Invariants

1. Same input transition produces identical receipt.
2. Modified event payload changes receipt.
3. Modified parent state changes receipt.
4. Kernel version changes invalidate compatibility.

## Proof Boundary

Cryptographic proof systems consume the receipt artifact,
not arbitrary implementation execution.

