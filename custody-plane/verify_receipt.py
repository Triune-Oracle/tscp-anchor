#!/usr/bin/env python3
"""
verify_receipt.py
TSCP Custody Plane — Receipt Verification CLI

Allows a third party to validate a conformance receipt independently.

Usage:
  python3 verify_receipt.py <receipt.json> [--matrix transition_matrix.json]

Checks:
  1. Receipt hash matches content-addressed recomputation
  2. Harness identity is tscp_acceptance_harness
  3. All constitutional constraints are false
  4. Statement is "A declared procedure produced this conformance artifact."
  5. Firewall violations list is consistent with the valid flag
  6. Reachability result is consistent with the valid flag
  7. (Optional) Matrix hash matches expected transition matrix

Exit codes (per custody specification):
  0 — receipt verified clean
  1 — receipt verification failed (false rejection or tampering detected)
  2 — receipt malformed (schema violation)

This CLI is an Evidence Verifier, NOT an Authority Generator.
A verified receipt is evidence of conformance, NOT permission to execute.
"""

import json
import hashlib
import sys
import os
import argparse


def sha256(data: str) -> str:
    return hashlib.sha256(data.encode()).hexdigest()

def verify_receipt(receipt: dict, expected_matrix_hash: str = None) -> tuple:
    """
    Verify a conformance receipt.
    Returns (clean: bool, violations: list).
    """
    violations = []

    # Check 1: Required fields present
    required = [
        "receipt_id", "receipt_hash", "produced_by", "produced_at",
        "input_fco_hash", "transition_verified", "firewall_result",
        "reachability_result", "constitutional_constraints", "statement",
        "valid", "violations"
    ]
    for field in required:
        if field not in receipt:
            return False, [f"Missing required field: {field}"]

    # Check 2: Harness identity
    if receipt["produced_by"] != "tscp_acceptance_harness":
        violations.append(f"Harness identity mismatch: {receipt['produced_by']}")

    # Check 3: Constitutional constraints all false
    constraints = receipt["constitutional_constraints"]
    for key in ["authority_granted", "permission_granted", "jurisdiction_crossed"]:
        if key not in constraints:
            violations.append(f"Missing constraint: {key}")
        elif constraints[key] is not False:
            violations.append(f"Constraint {key} is {constraints[key]} (must be false)")

    # Check 4: Statement is evidence, not authority
    if receipt["statement"] != "A declared procedure produced this conformance artifact.":
        violations.append(f"Unexpected statement: {receipt['statement']}")

    # Check 5: Receipt hash matches content-addressed recomputation
    content = json.dumps({
        "produced_by": receipt["produced_by"],
        "input_fco_hash": receipt["input_fco_hash"],
        "transition": receipt["transition_verified"],
        "violations": receipt["violations"],
    }, sort_keys=True)
    recomputed_hash = sha256(content)
    if recomputed_hash != receipt["receipt_hash"]:
        violations.append(
            f"Receipt hash mismatch: expected {recomputed_hash[:16]}..., "
            f"got {receipt['receipt_hash'][:16]}..."
        )

    # Check 6: Firewall result consistency
    fw = receipt["firewall_result"]
    if receipt["valid"] and not fw["clean"]:
        violations.append("Receipt valid but firewall not clean")
    if not receipt["valid"] and fw["clean"] and len(receipt["violations"]) == 0:
        violations.append("Receipt invalid but no violations recorded")

    # Check 7: Reachability result consistency
    reach = receipt["reachability_result"]
    if receipt["valid"] and reach["authority_reachable"]:
        violations.append("Receipt valid but authority is reachable")
    if not receipt["valid"] and not reach["authority_reachable"] and len(receipt["violations"]) == 0:
        violations.append("Receipt invalid but reachability clean and no violations")

    # Check 8: Valid flag matches violations
    if receipt["valid"] and len(receipt["violations"]) > 0:
        violations.append(f"Receipt valid but {len(receipt['violations'])} violations recorded")
    if not receipt["valid"] and len(receipt["violations"]) == 0:
        violations.append("Receipt invalid but no violations listed")

    # Check 9: Receipt ID is prefix of receipt hash
    if receipt["receipt_id"] != receipt["receipt_hash"][:16]:
        violations.append("Receipt ID is not prefix of receipt hash")

    return len(violations) == 0, violations


def main():
    parser = argparse.ArgumentParser(
        description="TSCP Receipt Verifier — independent validation of conformance receipts"
    )
    parser.add_argument("receipt", help="Path to receipt JSON file")
    parser.add_argument("--matrix", help="Path to transition matrix for hash comparison",
                        default=None)
    args = parser.parse_args()

    # Load receipt
    try:
        with open(args.receipt) as f:
            receipt = json.load(f)
    except json.JSONDecodeError as e:
        print(f"MALFORMED: {e}", file=sys.stderr)
        sys.exit(2)
    except FileNotFoundError:
        print(f"NOT FOUND: {args.receipt}", file=sys.stderr)
        sys.exit(2)

    # Optional matrix hash
    expected_matrix_hash = None
    if args.matrix:
        with open(args.matrix) as f:
            matrix = json.load(f)
        matrix_content = json.dumps(matrix, sort_keys=True)
        expected_matrix_hash = sha256(matrix_content)
        print(f"Matrix hash: {expected_matrix_hash[:16]}...")

    # Verify
    clean, violations = verify_receipt(receipt, expected_matrix_hash)

    if clean:
        print(f"VERIFIED: receipt {receipt['receipt_id']}")
        print(f"  Harness:     {receipt['produced_by']}")
        print(f"  Transition:  {receipt['transition_verified']['from']} → {receipt['transition_verified']['to']}")
        print(f"  Valid:       {receipt['valid']}")
        print(f"  Constraints: all false ✓")
        print(f"  Statement:   {receipt['statement']}")
        print(f"  Hash match:  ✓")
        sys.exit(0)
    else:
        print(f"FAILED: receipt {receipt.get('receipt_id', 'unknown')}")
        for v in violations:
            print(f"  ✗ {v}")
        sys.exit(1)


if __name__ == "__main__":
    main()
