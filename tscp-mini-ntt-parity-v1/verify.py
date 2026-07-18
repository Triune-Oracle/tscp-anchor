#!/usr/bin/env python3
"""
verify.py — TSCP Mini-NTT Parity Verifier

Loads pre-generated vectors from vectors/n8.json and vectors/n16.json
and independently re-computes every claim. Exits 0 on full pass, 1 on
any failure.

This is the acceptance gate. An implementation is compliant when
its outputs match these vectors exactly.

Usage:
  python verify.py
"""

import json
import sys
import hashlib
from pathlib import Path

# ── BabyBear field ────────────────────────────────────────────────────────────
P = 2013265921

def field_add(a, b): return (a + b) % P
def field_sub(a, b): return (a - b) % P
def field_mul(a, b): return (a * b) % P
def field_inv(a):    return pow(a, P - 2, P)

# ── Cooley-Tukey DIT NTT (identical to vector_generator.py) ──────────────────
def ntt(a, omega):
    n = len(a)
    a = list(a)
    j = 0
    for i in range(1, n):
        bit = n >> 1
        while j & bit:
            j ^= bit
            bit >>= 1
        j ^= bit
        if i < j:
            a[i], a[j] = a[j], a[i]
    length = 2
    while length <= n:
        w = pow(omega, n // length, P)
        for i in range(0, n, length):
            wn = 1
            for k in range(length // 2):
                u = a[i + k]
                v = (a[i + k + length // 2] * wn) % P
                a[i + k] = (u + v) % P
                a[i + k + length // 2] = (u - v) % P
                wn = (wn * w) % P
        length <<= 1
    return a

def intt(a, omega):
    n = len(a)
    omega_inv = field_inv(omega)
    result = ntt(a, omega_inv)
    n_inv = field_inv(n)
    return [(x * n_inv) % P for x in result]

# ── Deterministic input (must match vector_generator.py exactly) ──────────────
def deterministic_inputs(n, seed="TSCP-MINI-NTT-PARITY-V1"):
    return [int.from_bytes(hashlib.sha256(f"{seed}:{i}".encode()).digest()[:8], 'big') % P
            for i in range(n)]

# ── Individual checks ─────────────────────────────────────────────────────────
PASS = "  ✓ PASS"
FAIL = "  ✗ FAIL"

def check(label, condition, detail=""):
    status = PASS if condition else FAIL
    print(f"{status}  {label}" + (f"  [{detail}]" if detail else ""))
    return condition

def verify_vector(path):
    """Load a vector file and verify all claims independently."""
    with open(path) as f:
        vec = json.load(f)

    n        = vec["n"]
    omega    = vec["omega"]
    omega_inv = vec["omega_inv"]
    n_inv    = vec["n_inv"]
    inputs   = vec["inputs"]
    ntt_out  = vec["ntt_outputs"]
    intt_out = vec["intt_of_inputs"]
    label    = vec["_label"]

    print(f"\n{'─'*56}")
    print(f"  Vector: {label}  (n={n})")
    print(f"{'─'*56}")

    ok = True

    # 1. omega is a primitive n-th root of unity
    ok &= check(
        f"omega^{n} == 1 mod p",
        pow(omega, n, P) == 1
    )
    ok &= check(
        f"omega^{n//2} != 1 mod p  (primitive)",
        pow(omega, n // 2, P) != 1
    )

    # 2. omega_inv is the actual inverse
    ok &= check(
        "omega * omega_inv == 1 mod p",
        (omega * omega_inv) % P == 1
    )

    # 3. n_inv is the actual inverse of n
    ok &= check(
        f"{n} * n_inv == 1 mod p",
        (n * n_inv) % P == 1
    )

    # 4. Inputs match deterministic generation
    expected_inputs = deterministic_inputs(n)
    ok &= check(
        "inputs match deterministic seed",
        inputs == expected_inputs,
        detail="" if inputs == expected_inputs else f"first mismatch at index {next(i for i,(a,b) in enumerate(zip(inputs,expected_inputs)) if a!=b)}"
    )

    # 5. NTT outputs match independent computation
    computed_ntt = ntt(inputs, omega)
    ok &= check(
        "ntt_outputs match independent computation",
        ntt_out == computed_ntt,
        detail="" if ntt_out == computed_ntt else f"first mismatch at index {next(i for i,(a,b) in enumerate(zip(ntt_out,computed_ntt)) if a!=b)}"
    )

    # 6. INTT of inputs matches independent computation
    computed_intt = intt(inputs, omega)
    ok &= check(
        "intt_of_inputs matches independent computation",
        intt_out == computed_intt
    )

    # 7. Parity invariant: INTT(NTT(x)) == x
    recovered = intt(computed_ntt, omega)
    ok &= check(
        "INTT(NTT(x)) == x  (parity invariant)",
        recovered == inputs
    )

    # 8. Convolution theorem: NTT then INTT are mutual inverses
    roundtrip = ntt(intt(inputs, omega), omega)
    ok &= check(
        "NTT(INTT(x)) == x  (convolution theorem)",
        roundtrip == inputs
    )

    # 9. Linearity: NTT(a+b) == NTT(a) + NTT(b)
    inputs_b = deterministic_inputs(n, seed="TSCP-MINI-NTT-PARITY-V1-B")
    ntt_a = ntt(inputs, omega)
    ntt_b = ntt(inputs_b, omega)
    ntt_sum = ntt([(inputs[i] + inputs_b[i]) % P for i in range(n)], omega)
    expected_linear = [(ntt_a[i] + ntt_b[i]) % P for i in range(n)]
    ok &= check(
        "NTT(a+b) == NTT(a)+NTT(b)  (linearity)",
        ntt_sum == expected_linear
    )

    # 10. Zero vector invariant: NTT([0,...,0]) == [0,...,0]
    zero = [0] * n
    ok &= check(
        "NTT(zero) == zero",
        ntt(zero, omega) == zero
    )

    # 11. Schema check
    ok &= check(
        "_schema == 'tscp-ntt-parity-vector-v1'",
        vec.get("_schema") == "tscp-ntt-parity-vector-v1"
    )

    return ok

def verify_sha256sums():
    """Verify SHA256SUMS file if present."""
    sums_path = Path("SHA256SUMS")
    if not sums_path.exists():
        print(f"\n{FAIL}  SHA256SUMS not found")
        return False

    print(f"\n{'─'*56}")
    print("  SHA256SUMS integrity")
    print(f"{'─'*56}")

    ok = True
    for line in sums_path.read_text().strip().splitlines():
        parts = line.split("  ", 1)
        if len(parts) != 2:
            continue
        expected_hex, fname = parts
        fpath = Path(fname)
        if not fpath.exists():
            print(f"{FAIL}  {fname}  [file not found]")
            ok = False
            continue
        actual_hex = hashlib.sha256(fpath.read_bytes()).hexdigest()
        match = actual_hex == expected_hex
        ok &= check(f"{fname}", match,
                    detail="" if match else f"expected {expected_hex[:12]}..., got {actual_hex[:12]}...")
    return ok

# ── Main ──────────────────────────────────────────────────────────────────────
def main():
    print("=" * 56)
    print("  TSCP Mini-NTT Parity Verifier")
    print("  Field: BabyBear (p = 2013265921)")
    print("=" * 56)

    all_ok = True

    for vec_path in ["vectors/n8.json", "vectors/n16.json"]:
        if not Path(vec_path).exists():
            print(f"\n{FAIL}  {vec_path} not found — run vector_generator.py first")
            all_ok = False
            continue
        all_ok &= verify_vector(vec_path)

    all_ok &= verify_sha256sums()

    print()
    print("=" * 56)
    if all_ok:
        print("  RESULT: ALL CHECKS PASSED")
        print("  Status: VERIFICATION_PACKAGE_PASS")
        print("  This output is the first real evidence object.")
    else:
        print("  RESULT: VERIFICATION FAILED")
        print("  Status: VERIFICATION_PACKAGE_FAIL")
        sys.exit(1)
    print("=" * 56)

if __name__ == "__main__":
    main()
