# Identity Seal: tscp-serialization-v0.1

**Status:** VERIFIED
**Sealed:** 2026-07-19T15:45:00Z
**Authority:** Cartilage-Stairwells Verification Authority

## Verification Result

All seven verification predicates defined in CONTRACT.md have been satisfied.

| # | Predicate | Verdict |
|---|---|---|
| 1 | Schema Compliance | PASS |
| 2 | Canonical Encoding Correctness | PASS |
| 3 | Hash Stability | PASS |
| 4 | Mutation Sensitivity | PASS |
| 5 | Context Isolation | PASS |
| 6 | Toolchain Reproducibility | PASS |
| 7 | Authority Boundary Preservation | PASS |

## Environment

- **rustc:** 1.97.1 (8bab26f4f 2026-07-14)
- **cargo:** 1.97.1 (c980f4866 2026-06-30)
- **Platform:** Linux x86_64 (gvisor)
- **Kernel:** 4.19.0-gvisor

## Identity Anchors

| Artifact | SHA-256 |
|---|---|
| Verification run log | `4db5f5919603926b3b4ed9e8792c4233a1a3dcb3d9220663f7a6db76b64f6019` |
| Test suite (serialization_conformance.rs) | `d760e03958199c4afd9d0fda0c5638ef42efe3cbedab5028a27cf458e1f6ebd8` |
| serialization.rs | `676563d082490dcdd69787446da2e5279917ef97f97047371be817b9ea4da8b6` |
| types.rs | `c538f9440f94ab64652ef7cc0f06dc1ba4e08d95ecf09a38aa307bd7258f468e` |

## Change note (2026-07-19)

Test suite updated to fix two remaining issues identified in CI review:

1. **cargo fmt compliance:** All test functions reformatted to nightly rustfmt
   style — trailing commas, consistent assert spacing, no alignment padding.

2. **Custody rejection test correctness:** `test_custody_expression_blocked`
   now uses binary CBOR stream mutation rather than `serde_cbor::Value` map
   construction. The valid serialized receipt (0xA5 map header = 5 fields) is
   patched to 0xA6 (6 fields) and a CBOR-encoded ("custody", "approve") pair
   is appended. This exercises `deny_unknown_fields` against a structurally
   valid CBOR payload with the correct field encoding — not a type-mismatch
   rejection. The test now proves the invariant rather than passing for the
   wrong reason.

## Scope

This seal certifies artifact identity verification only.
It does NOT imply VERIFIED\_CUSTODY. Custody carries independent predicates
and is not inherited from artifact identity verification.

## Transition

```
FROZEN_SPECIFICATION
        ↓
  Verification execution (12/12 tests PASS)
        ↓
VERIFICATION_PACKAGE_PASS
        ↓
VERIFIED_ARTIFACT_IDENTITY ← sealed 2026-07-19T15:45:00Z
```

---

*Sealed by Cartilage-Stairwells Verification Authority*
*tscp-serialization-v0.1-verification*
