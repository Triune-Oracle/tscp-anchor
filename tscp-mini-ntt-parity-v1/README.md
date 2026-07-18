# tscp-mini-ntt-parity-v1

**First real evidence object for the TSCP arithmetic layer.**

This package produces and verifies reference NTT test vectors over the BabyBear prime field.
It is the acceptance gate for cross-implementation parity: any implementation
(Rust/AVX-512, WASM, software fallback) must reproduce these vectors exactly.

## Field

BabyBear: `p = 2013265921 = 15 × 2²⁷ + 1`

Chosen for its 2-adicity (27), making it efficient for power-of-2 NTTs up to length 2²⁷.
Used by Plonky3 and related STARK backends.

## What this proves

| Invariant | Verified |
|---|---|
| `omega^n == 1 mod p` | ✓ |
| `omega^(n/2) != 1 mod p` (primitive) | ✓ |
| `INTT(NTT(x)) == x` (parity) | ✓ |
| `NTT(INTT(x)) == x` (convolution theorem) | ✓ |
| `NTT(a+b) == NTT(a)+NTT(b)` (linearity) | ✓ |
| `NTT(zero) == zero` (zero invariant) | ✓ |
| Inputs match deterministic seed | ✓ |
| SHA-256 digest integrity | ✓ |

## Structure

```
tscp-mini-ntt-parity-v1/
  constants.json          — field constants with verification proofs
  vector_generator.py     — generates reference vectors and verifies invariants
  verify.py               — independent re-computation verifier (acceptance gate)
  vectors/
    n8.json               — length-8 NTT/INTT reference vectors
    n16.json              — length-16 NTT/INTT reference vectors
  SHA256SUMS              — artifact integrity manifest
  verification.log        — execution record (first evidence object)
  README.md               — this file
```

## Usage

```bash
# Generate vectors (verifies all invariants during generation)
python vector_generator.py

# Verify independently (re-computes everything from scratch)
python verify.py

# Check artifact integrity
sha256sum -c SHA256SUMS
```

Expected output from `python verify.py`:
```
RESULT: ALL CHECKS PASSED
Status: VERIFICATION_PACKAGE_PASS
```

## No external dependencies

Pure Python 3 standard library only. `hashlib`, `json`, `pathlib`.

## Artifact structure (TSCP evidence model)

```
TSCP Artifact
│
├── Code
│   └── (butterfly_avx512.rs — upstream, not in this package)
│
├── Evidence                      ← this package produces this layer
│   ├── vectors/n8.json
│   ├── vectors/n16.json
│   ├── SHA256SUMS
│   └── verification.log
│
├── Metadata
│   └── (shark_annotation.jsonld — next milestone)
│
└── Policy
    └── (ilh_policy.rego — next milestone)
```

## Next milestone

Attach the S.H.A.R.K./ILH metadata layer:

```json
{
  "@context": "https://example.org/shark/v1/context.jsonld",
  "contextHash": "sha256:...",
  "contextVersion": "1.0.0",
  "artifact": "tscp-mini-ntt-parity-v1",
  "evidenceRecord": "verification.log"
}
```

Then wire the OPA policy for transformation permissions:

```rego
deny_refactor[msg] {
    input.Confidence < 0.90
    msg := "Manual review required before refactorization"
}
```

## Execution record

`verification.log` contains the machine-readable record of this execution:
- SHA-256 of every artifact
- 24/24 checks passed
- Status: `VERIFICATION_PACKAGE_PASS`

---

*Part of the TSCP arithmetic layer verification chain.*  
*Date: 2026-07-18*
