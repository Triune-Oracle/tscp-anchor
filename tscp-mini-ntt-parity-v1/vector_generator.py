#!/usr/bin/env python3
"""
vector_generator.py — TSCP Mini-NTT Parity Test Vector Generator

Produces reference NTT/INTT vectors over the BabyBear prime field.
All arithmetic is pure Python — no external dependencies.

Outputs:
  vectors/n8.json   — length-8 NTT and INTT test vectors
  vectors/n16.json  — length-16 NTT and INTT test vectors
  SHA256SUMS        — digest file for artifact integrity

Usage:
  python vector_generator.py

Evidence contract:
  These vectors serve as the ground-truth reference for cross-implementation
  parity verification. Any implementation (Rust/AVX-512, WASM, software fallback)
  must reproduce these values exactly to pass parity.
"""

import json
import hashlib
import os
import sys
from pathlib import Path

# ── BabyBear field parameters ────────────────────────────────────────────────
P = 2013265921          # 15 * 2^27 + 1
PRIMITIVE_ROOT = 31

def field_add(a, b): return (a + b) % P
def field_sub(a, b): return (a - b) % P
def field_mul(a, b): return (a * b) % P
def field_pow(base, exp): return pow(base, exp, P)
def field_inv(a): return pow(a, P - 2, P)  # Fermat's little theorem

# ── Verify field constants ────────────────────────────────────────────────────
def verify_constants():
    """Verify all constants in constants.json against computed values."""
    with open("constants.json") as f:
        c = json.load(f)

    errors = []

    # Prime check
    assert c["prime"] == P, "prime mismatch"

    # Primitive root check: g^((p-1)/2) != 1 mod p (i.e., g is not a QR)
    if field_pow(PRIMITIVE_ROOT, (P - 1) // 2) == 1:
        errors.append("PRIMITIVE_ROOT fails QR check")

    # omega_n8
    computed_w8 = field_pow(PRIMITIVE_ROOT, (P - 1) // 8)
    if c["omega_n8"] != computed_w8:
        errors.append(f"omega_n8: expected {computed_w8}, got {c['omega_n8']}")
    if field_pow(computed_w8, 8) != 1:
        errors.append("omega_n8^8 != 1")
    if field_pow(computed_w8, 4) == 1:
        errors.append("omega_n8^4 == 1 (not a primitive 8th root)")

    # omega_n16
    computed_w16 = field_pow(PRIMITIVE_ROOT, (P - 1) // 16)
    if c["omega_n16"] != computed_w16:
        errors.append(f"omega_n16: expected {computed_w16}, got {c['omega_n16']}")
    if field_pow(computed_w16, 16) != 1:
        errors.append("omega_n16^16 != 1")
    if field_pow(computed_w16, 8) == 1:
        errors.append("omega_n16^8 == 1 (not a primitive 16th root)")

    # inverses
    if field_mul(c["omega_n8"], c["omega_n8_inv"]) != 1:
        errors.append("omega_n8 * omega_n8_inv != 1")
    if field_mul(c["omega_n16"], c["omega_n16_inv"]) != 1:
        errors.append("omega_n16 * omega_n16_inv != 1")
    if field_mul(8, c["n8_inv"]) % P != 1:
        errors.append("8 * n8_inv != 1 mod p")
    if field_mul(16, c["n16_inv"]) % P != 1:
        errors.append("16 * n16_inv != 1 mod p")

    if errors:
        print("CONSTANT VERIFICATION FAILED:")
        for e in errors:
            print(f"  - {e}")
        sys.exit(1)

    print("constants.json: all values verified ✓")

# ── Cooley-Tukey DIT NTT (decimation-in-time) ────────────────────────────────
def ntt(a, omega):
    """
    In-place Cooley-Tukey DIT NTT over BabyBear.
    Input/output in natural order (not bit-reversed).
    omega must be a primitive n-th root of unity where n = len(a).
    """
    n = len(a)
    assert n & (n - 1) == 0, "n must be a power of 2"
    a = list(a)

    # Bit-reversal permutation
    j = 0
    for i in range(1, n):
        bit = n >> 1
        while j & bit:
            j ^= bit
            bit >>= 1
        j ^= bit
        if i < j:
            a[i], a[j] = a[j], a[i]

    # Butterfly stages
    length = 2
    while length <= n:
        w = field_pow(omega, n // length)  # twiddle factor for this stage
        for i in range(0, n, length):
            wn = 1
            for k in range(length // 2):
                u = a[i + k]
                v = field_mul(a[i + k + length // 2], wn)
                a[i + k] = field_add(u, v)
                a[i + k + length // 2] = field_sub(u, v)
                wn = field_mul(wn, w)
        length <<= 1

    return a

def intt(a, omega):
    """
    Inverse NTT: INTT(a) = (1/n) * NTT(a, omega^{-1})
    """
    n = len(a)
    omega_inv = field_inv(omega)
    result = ntt(a, omega_inv)
    n_inv = field_inv(n)
    return [field_mul(x, n_inv) for x in result]

# ── Deterministic input generation ───────────────────────────────────────────
def deterministic_inputs(n, seed="TSCP-MINI-NTT-PARITY-V1"):
    """
    Generate n deterministic field elements from a seed string.
    Uses SHA-256 in counter mode. Values are reduced mod P.
    """
    inputs = []
    for i in range(n):
        digest = hashlib.sha256(f"{seed}:{i}".encode()).digest()
        val = int.from_bytes(digest[:8], 'big') % P
        inputs.append(val)
    return inputs

# ── Parity test: NTT then INTT must recover input ────────────────────────────
def verify_parity(original, recovered, label):
    if original != recovered:
        mismatches = [(i, original[i], recovered[i])
                      for i in range(len(original)) if original[i] != recovered[i]]
        print(f"PARITY FAILURE for {label}:")
        for idx, exp, got in mismatches:
            print(f"  index {idx}: expected {exp}, got {got}")
        sys.exit(1)
    print(f"{label}: INTT(NTT(x)) == x ✓  (parity verified)")

# ── Linearity test: NTT(a+b) == NTT(a) + NTT(b) ─────────────────────────────
def verify_linearity(inputs_a, inputs_b, omega, label):
    ntt_a = ntt(inputs_a, omega)
    ntt_b = ntt(inputs_b, omega)
    ntt_sum = ntt([field_add(inputs_a[i], inputs_b[i]) for i in range(len(inputs_a))], omega)
    expected = [field_add(ntt_a[i], ntt_b[i]) for i in range(len(ntt_a))]
    if ntt_sum != expected:
        print(f"LINEARITY FAILURE for {label}")
        sys.exit(1)
    print(f"{label}: NTT(a+b) == NTT(a)+NTT(b) ✓  (linearity verified)")

# ── Build a vector record ─────────────────────────────────────────────────────
def build_vector(n, omega, omega_inv, n_inv_val, label):
    inputs = deterministic_inputs(n)
    ntt_out = ntt(inputs, omega)
    intt_out = intt(inputs, omega)  # This is INTT of the *original* inputs

    # The canonical check: INTT(NTT(x)) == x
    recovered = intt(ntt_out, omega)
    verify_parity(inputs, recovered, label)

    # Linearity check
    inputs_b = deterministic_inputs(n, seed="TSCP-MINI-NTT-PARITY-V1-B")
    verify_linearity(inputs, inputs_b, omega, label)

    return {
        "_schema": "tscp-ntt-parity-vector-v1",
        "_label": label,
        "_date": "2026-07-18",
        "_field": "BabyBear (p = 2013265921 = 15 * 2^27 + 1)",
        "_algorithm": "Cooley-Tukey DIT NTT, natural-order input, natural-order output",
        "_seed": "TSCP-MINI-NTT-PARITY-V1",
        "n": n,
        "omega": omega,
        "omega_inv": omega_inv,
        "n_inv": n_inv_val,
        "inputs": inputs,
        "ntt_outputs": ntt_out,
        "intt_of_inputs": intt_out,
        "invariants_verified": {
            "parity": True,
            "linearity": True
        }
    }

# ── SHA-256 digest helper ─────────────────────────────────────────────────────
def sha256_file(path):
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        h.update(f.read())
    return h.hexdigest()

# ── Main ──────────────────────────────────────────────────────────────────────
def main():
    print("=" * 60)
    print("  TSCP Mini-NTT Parity Vector Generator")
    print("  Field: BabyBear (p = 2013265921)")
    print("=" * 60)
    print()

    # Load and verify constants
    verify_constants()
    print()

    with open("constants.json") as f:
        c = json.load(f)

    os.makedirs("vectors", exist_ok=True)

    # n=8 vectors
    print("Generating n=8 vectors...")
    vec8 = build_vector(
        n=8,
        omega=c["omega_n8"],
        omega_inv=c["omega_n8_inv"],
        n_inv_val=c["n8_inv"],
        label="n8"
    )
    path8 = Path("vectors/n8.json")
    path8.write_text(json.dumps(vec8, indent=2))
    print(f"  Written: {path8}")
    print()

    # n=16 vectors
    print("Generating n=16 vectors...")
    vec16 = build_vector(
        n=16,
        omega=c["omega_n16"],
        omega_inv=c["omega_n16_inv"],
        n_inv_val=c["n16_inv"],
        label="n16"
    )
    path16 = Path("vectors/n16.json")
    path16.write_text(json.dumps(vec16, indent=2))
    print(f"  Written: {path16}")
    print()

    # SHA256SUMS
    files_to_digest = [
        "constants.json",
        "vectors/n8.json",
        "vectors/n16.json",
        "vector_generator.py",
        "verify.py",
    ]
    sums_lines = []
    for fname in files_to_digest:
        if Path(fname).exists():
            digest = sha256_file(fname)
            sums_lines.append(f"{digest}  {fname}")
            print(f"  SHA-256 {fname}: {digest[:16]}...")

    sums_path = Path("SHA256SUMS")
    sums_path.write_text("\n".join(sums_lines) + "\n")
    print(f"\n  Written: SHA256SUMS ({len(sums_lines)} entries)")

    print()
    print("=" * 60)
    print("  Generation complete. All invariants verified.")
    print("  Next: python verify.py && sha256sum -c SHA256SUMS")
    print("=" * 60)

if __name__ == "__main__":
    main()
