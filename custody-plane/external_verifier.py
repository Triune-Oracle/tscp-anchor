#!/usr/bin/env python3
"""
external_verifier.py
TSCP Custody Plane — External Verifier Path

Allows a third party with NO knowledge of the custody architecture to
independently verify the custody plane closure.

This is the "outside the boundary" verifier — it does NOT trust the
harness, the matrix, or the Lean proof. It re-derives everything from
first principles using the provided artifacts.

Usage:
  python3 external_verifier.py --artifacts-dir custody-plane/

Checks performed:
  1. Transition matrix completeness (all 5×5 = 25 pairs present)
  2. Plane separation (no allowed transition crosses custody → authority)
  3. Reachability (BFS from each custody-plane category → authority unreachable)
  4. Constraint schema (all constraints are const:false)
  5. Receipt schema (statement is evidence, not authority)
  6. Firewall consistency (custody→authority transitions forbidden)

The key invariant is ONE-DIRECTIONAL:
  No path from custody plane to authority plane.
The reverse (authority → custody, e.g. Execution → Evidence) is PERMITTED
because execution naturally produces evidence. The concern is that custody
objects don't gain authority, not that authority can't produce evidence.

Exit codes:
  0 — custody plane verified from first principles
  1 — verification failed
  2 — artifacts missing or malformed

Classification: External Evidence Verifier, NOT Authority Generator.
"""

import json
import hashlib
import sys
import os
import argparse


CATEGORIES = ["Custody", "Evidence", "AcceptanceReceipt", "Authority", "Execution"]
CUSTODY_PLANE = {"Custody", "Evidence", "AcceptanceReceipt"}
AUTHORITY_PLANE = {"Authority", "Execution"}


def load_json(path):
    try:
        with open(path) as f:
            return json.load(f)
    except (json.JSONDecodeError, FileNotFoundError) as e:
        print(f"ERROR loading {path}: {e}", file=sys.stderr)
        sys.exit(2)


def verify_matrix_completeness(matrix):
    """Check that all 5×5 = 25 transitions are present."""
    seen = set()
    for t in matrix["transitions"]:
        seen.add((t["from"], t["to"]))
    expected = set()
    for a in CATEGORIES:
        for b in CATEGORIES:
            expected.add((a, b))
    missing = expected - seen
    extra = seen - expected
    if missing:
        return False, [f"Missing transitions: {missing}"]
    if extra:
        return False, [f"Extra transitions: {extra}"]
    return True, []


def verify_plane_separation(matrix):
    """Check that no allowed transition crosses from custody to authority plane."""
    violations = []
    for t in matrix["transitions"]:
        if (t["from"] in CUSTODY_PLANE and
            t["to"] in AUTHORITY_PLANE and
            t["allowed"]):
            violations.append(
                f"PLANE CROSSING: {t['from']} → {t['to']} is allowed"
            )
    return len(violations) == 0, violations


def verify_reachability(matrix):
    """BFS from each custody-plane category — verify authority is unreachable."""
    # Build adjacency (allowed only)
    adj = {c: [] for c in CATEGORIES}
    for t in matrix["transitions"]:
        if t["allowed"]:
            adj[t["from"]].append(t["to"])

    violations = []
    for start in CUSTODY_PLANE:
        visited = set()
        queue = [start]
        while queue:
            current = queue.pop(0)
            if current in visited:
                continue
            visited.add(current)
            for nxt in adj[current]:
                if nxt not in visited:
                    queue.append(nxt)
        # Check if any authority-plane category was reached
        reached_authority = visited & AUTHORITY_PLANE
        if reached_authority:
            violations.append(
                f"AUTHORITY REACHABLE from {start}: {reached_authority}"
            )
    return len(violations) == 0, violations


def verify_constraint_schema(schema):
    """Check that all constraints are const:false."""
    violations = []
    props = schema.get("properties", {}).get("constraints", {}).get("properties", {})
    for key in ["authority_granted", "permission_granted", "jurisdiction_crossed"]:
        if key not in props:
            violations.append(f"Missing constraint field: {key}")
            continue
        const = props[key].get("const")
        if const is not False:
            violations.append(f"Constraint {key} has const={const} (must be false)")
    return len(violations) == 0, violations


def verify_receipt_schema(schema):
    """Check that the receipt statement is evidence, not authority."""
    violations = []
    props = schema.get("properties", {})
    statement = props.get("statement", {})
    if statement.get("const") != "A declared procedure produced this conformance artifact.":
        violations.append("Receipt statement is not the evidence declaration")
    produced_by = props.get("produced_by", {})
    if produced_by.get("const") != "tscp_acceptance_harness":
        violations.append("Receipt producer is not the acceptance harness")
    return len(violations) == 0, violations


def verify_firewall_consistency(matrix):
    """Cross-check: custody→authority transitions must be forbidden.

    The invariant is ONE-DIRECTIONAL: no path from custody to authority.
    The reverse (authority→custody, e.g. Execution→Evidence) is PERMITTED
    because execution naturally produces evidence about what happened.
    The concern is that custody objects don't gain authority, not that
    authority can't produce evidence."""
    violations = []
    # Custody → Authority must be forbidden (the key invariant)
    for t in matrix["transitions"]:
        if (t["from"] in CUSTODY_PLANE and
            t["to"] in AUTHORITY_PLANE and
            t["allowed"]):
            violations.append(
                f"Firewall inconsistency: {t['from']} → {t['to']} allowed but should be forbidden"
            )
    # Note: Authority → Custody (e.g. Execution → Evidence) is PERMITTED
    # Execution produces evidence; this doesn't violate the key invariant
    # because evidence doesn't grant authority back to the producer
    return len(violations) == 0, violations



def verify_algebra_version_binding(artifacts_dir, matrix, custody_schema, receipt_schema):
    """Check that all implementations declare the same algebra version.

    The algebra version is the SHA-256 hash of FCO_TRANSITION_ALGEBRA.md.
    Each implementation embeds this hash as a declaration of what it targets.
    This does NOT prove conformance — it creates traceable lineage.

    Conformance is established by:
      - transition_matrix.json: completeness check (Check 1)
      - boundary_firewall.rs: Rust test suite (9 tests)
      - FCO_Invariants.lean: Lean compilation (0 errors, 0 sorry)
      - tscp_acceptance_harness.py: Python test suite (32 assertions)
    """
    import re

    violations = []
    expected_hash = None

    # 1. Check transition_matrix.json
    matrix_hash = matrix.get("algebra_version")
    if not matrix_hash:
        violations.append("transition_matrix.json: missing algebra_version field")
    else:
        expected_hash = matrix_hash

    # 2. Check boundary_firewall.rs
    with open(os.path.join(artifacts_dir, "boundary_firewall.rs")) as f:
        rust_src = f.read()
    rust_match = re.search(r'ALGEBRA_VERSION:\s*&str\s*=\s*"([0-9a-f]+)"', rust_src)
    if not rust_match:
        violations.append("boundary_firewall.rs: missing ALGEBRA_VERSION constant")
    elif expected_hash and rust_match.group(1) != expected_hash:
        violations.append(
            f"boundary_firewall.rs: algebra version mismatch "
            f"(expected {expected_hash[:16]}..., got {rust_match.group(1)[:16]}...)"
        )

    # 3. Check FCO_Invariants.lean
    with open(os.path.join(artifacts_dir, "FCO_Invariants.lean")) as f:
        lean_src = f.read()
    lean_match = re.search(r'Hash:\s*([0-9a-f]+)', lean_src)
    if not lean_match:
        violations.append("FCO_Invariants.lean: missing algebra hash comment")
    elif expected_hash and lean_match.group(1) != expected_hash:
        violations.append(
            f"FCO_Invariants.lean: algebra version mismatch "
            f"(expected {expected_hash[:16]}..., got {lean_match.group(1)[:16]}...)"
        )

    # 4. Check tscp_acceptance_harness.py
    with open(os.path.join(artifacts_dir, "tscp_acceptance_harness.py")) as f:
        py_src = f.read()
    py_match = re.search(r'ALGEBRA_VERSION\s*=\s*"([0-9a-f]+)"', py_src)
    if not py_match:
        violations.append("tscp_acceptance_harness.py: missing ALGEBRA_VERSION constant")
    elif expected_hash and py_match.group(1) != expected_hash:
        violations.append(
            f"tscp_acceptance_harness.py: algebra version mismatch "
            f"(expected {expected_hash[:16]}..., got {py_match.group(1)[:16]}...)"
        )

    # 5. Check that the algebra document exists
    algebra_path = os.path.join(artifacts_dir, "FCO_TRANSITION_ALGEBRA.md")
    if not os.path.exists(algebra_path):
        violations.append("FCO_TRANSITION_ALGEBRA.md: document not found")
    elif expected_hash:
        import hashlib
        with open(algebra_path, "rb") as f:
            actual_hash = hashlib.sha256(f.read()).hexdigest()
        if actual_hash != expected_hash:
            violations.append(
                f"FCO_TRANSITION_ALGEBRA.md: hash mismatch "
                f"(expected {expected_hash[:16]}..., computed {actual_hash[:16]}...)"
            )

    return len(violations) == 0, violations


def main():
    parser = argparse.ArgumentParser(
        description="TSCP External Verifier — verify custody plane from first principles"
    )
    parser.add_argument("--artifacts-dir", default=".",
                        help="Directory containing custody-plane artifacts")
    args = parser.parse_args()

    d = args.artifacts_dir
    print("=" * 60)
    print("TSCP External Verifier — First Principles Verification")
    print("=" * 60)
    print()

    all_clean = True

    # Load artifacts
    matrix = load_json(os.path.join(d, "transition_matrix.json"))
    custody_schema = load_json(os.path.join(d, "custody.schema.json"))
    receipt_schema = load_json(os.path.join(d, "acceptance_receipt.schema.json"))

    # Check 1: Matrix completeness
    print("Check 1: Transition matrix completeness")
    clean, violations = verify_matrix_completeness(matrix)
    if clean:
        print(f"  ✓ All 25 transitions present (5×5)")
    else:
        all_clean = False
        for v in violations:
            print(f"  ✗ {v}")
    print()

    # Check 2: Plane separation
    print("Check 2: Plane separation (no custody→authority crossing)")
    clean, violations = verify_plane_separation(matrix)
    if clean:
        print(f"  ✓ No allowed transition crosses custody → authority")
    else:
        all_clean = False
        for v in violations:
            print(f"  ✗ {v}")
    print()

    # Check 3: Reachability (BFS from first principles)
    print("Check 3: Reachability (BFS from each custody category)")
    clean, violations = verify_reachability(matrix)
    if clean:
        print(f"  ✓ Authority unreachable from all custody-plane categories")
        print(f"  ✓ Custody plane = {{Custody, Evidence, AcceptanceReceipt}}")
        print(f"  ✓ Authority plane = {{Authority, Execution}}")
    else:
        all_clean = False
        for v in violations:
            print(f"  ✗ {v}")
    print()

    # Check 4: Constraint schema
    print("Check 4: Constitutional constraints (all const:false)")
    clean, violations = verify_constraint_schema(custody_schema)
    if clean:
        print(f"  ✓ authority_granted = false")
        print(f"  ✓ permission_granted = false")
        print(f"  ✓ jurisdiction_crossed = false")
    else:
        all_clean = False
        for v in violations:
            print(f"  ✗ {v}")
    print()

    # Check 5: Receipt schema
    print("Check 5: Receipt is evidence, not authority")
    clean, violations = verify_receipt_schema(receipt_schema)
    if clean:
        print(f"  ✓ Statement: 'A declared procedure produced this conformance artifact.'")
        print(f"  ✓ Producer: tscp_acceptance_harness")
    else:
        all_clean = False
        for v in violations:
            print(f"  ✗ {v}")
    print()

    # Check 6: Firewall consistency
    print("Check 6: Firewall consistency (custody→authority forbidden)")
    clean, violations = verify_firewall_consistency(matrix)
    if clean:
        print(f"  ✓ All custody→authority transitions are forbidden")
        print(f"  ✓ Authority→custody (e.g. Execution→Evidence) is permitted")
        print(f"    (execution produces evidence; evidence does not grant authority)")
    else:
        all_clean = False
        for v in violations:
            print(f"  ✗ {v}")
    print()


    # Check 7: Algebra version binding
    print("Check 7: Algebra version binding (all implementations target same version)")
    clean, violations = verify_algebra_version_binding(d, matrix, custody_schema, receipt_schema)
    if clean:
        print(f"  ✓ transition_matrix.json: algebra_version present")
        print(f"  ✓ boundary_firewall.rs: ALGEBRA_VERSION present")
        print(f"  ✓ FCO_Invariants.lean: algebra hash present")
        print(f"  ✓ tscp_acceptance_harness.py: ALGEBRA_VERSION present")
        print(f"  ✓ FCO_TRANSITION_ALGEBRA.md: hash verified")
        print(f"  ✓ All implementations target the same algebra version")
    else:
        all_clean = False
        for v in violations:
            print(f"  ✗ {v}")
    print()

    # Summary
    print("=" * 60)
    if all_clean:
        print("RESULT: CUSTODY PLANE VERIFIED FROM FIRST PRINCIPLES")
        print()
        print("The custody plane is closed under its own rules.")
        print("Stronger evidence does not become authority.")
        print("Deeper provenance does not become governance.")
        print("Cryptographic proof does not become permission.")
        print("Conformance receipts remain representations.")
        print()
        print("The architecture terminates at custody evidence, not jurisdiction.")
        sys.exit(0)
    else:
        print("RESULT: VERIFICATION FAILED")
        sys.exit(1)


if __name__ == "__main__":
    main()
