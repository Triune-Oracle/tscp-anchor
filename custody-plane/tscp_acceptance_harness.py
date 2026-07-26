#!/usr/bin/env python3
"""
tscp_acceptance_harness.py
TSCP Custody Plane — Acceptance Harness

Classification: Evidence Generator, NOT an Authority Generator.

Consumes a custody object and transition matrix, runs the firewall check
(Custody → AcceptanceReceipt), validates recursive FCO boundary compliance,
checks reachability (no path to authority from any custody-plane category),
and produces a content-addressed receipt.

The receipt records: "A declared procedure produced this conformance artifact."
It does NOT record: "This artifact deserves execution."

All constitutional constraints are const:false — negative assertions that
authority was NOT granted, permission was NOT granted, jurisdiction was
NOT crossed.
"""

# ── Algebra Version Binding ─────────────────────────────────────────────────
# This implementation targets FCO_TRANSITION_ALGEBRA.md
# Hash: 53fb4b5a093f7539587be2fc7703f482ac0f5c23c9bbdc89f9ef7614b7df7cda
# The external verifier checks that this hash matches the algebra document.
# This binding does NOT prove conformance — it declares the target.
# Conformance is established by the test suite, not by this constant.
ALGEBRA_VERSION = "53fb4b5a093f7539587be2fc7703f482ac0f5c23c9bbdc89f9ef7614b7df7cda"


import json
import hashlib
from datetime import datetime, timezone
from typing import Optional


# ── Transition Matrix (mirrors boundary_firewall.rs) ──────────────────────

CATEGORIES = {
    "Custody": "custody",
    "Evidence": "custody",
    "AcceptanceReceipt": "custody",
    "Authority": "authority",
    "Execution": "authority",
}

TRANSITIONS = [
    {"from": "Custody",           "to": "Custody",          "allowed": True},
    {"from": "Custody",           "to": "Evidence",          "allowed": True},
    {"from": "Custody",           "to": "AcceptanceReceipt", "allowed": True},
    {"from": "Custody",           "to": "Authority",         "allowed": False},
    {"from": "Custody",           "to": "Execution",         "allowed": False},
    {"from": "Evidence",          "to": "Evidence",          "allowed": True},
    {"from": "Evidence",          "to": "AcceptanceReceipt", "allowed": True},
    {"from": "Evidence",          "to": "Authority",         "allowed": False},
    {"from": "Evidence",          "to": "Execution",         "allowed": False},
    {"from": "Evidence",          "to": "Custody",            "allowed": False},
    {"from": "AcceptanceReceipt", "to": "AcceptanceReceipt", "allowed": True},
    {"from": "AcceptanceReceipt", "to": "Evidence",           "allowed": True},
    {"from": "AcceptanceReceipt", "to": "Custody",            "allowed": False},
    {"from": "AcceptanceReceipt", "to": "Authority",         "allowed": False},
    {"from": "AcceptanceReceipt", "to": "Execution",         "allowed": False},
    {"from": "Authority",         "to": "Authority",          "allowed": True},
    {"from": "Authority",         "to": "Execution",         "allowed": True},
    {"from": "Authority",         "to": "Custody",            "allowed": False},
    {"from": "Authority",         "to": "Evidence",           "allowed": False},
    {"from": "Authority",         "to": "AcceptanceReceipt",  "allowed": False},
    {"from": "Execution",         "to": "Execution",         "allowed": True},
    {"from": "Execution",         "to": "Evidence",           "allowed": True},
    {"from": "Execution",         "to": "Custody",            "allowed": False},
    {"from": "Execution",         "to": "Authority",         "allowed": False},
    {"from": "Execution",         "to": "AcceptanceReceipt",  "allowed": False},
]

CUSTODY_PLANE = {"Custody", "Evidence", "AcceptanceReceipt"}
AUTHORITY_PLANE = {"Authority", "Execution"}


# ── Firewall ──────────────────────────────────────────────────────────────

def check_transition(from_cat: str, to_cat: str) -> Optional[bool]:
    """Returns True if allowed, False if forbidden, None if not in matrix."""
    for t in TRANSITIONS:
        if t["from"] == from_cat and t["to"] == to_cat:
            return t["allowed"]
    return None

def is_known_category(cat: str) -> bool:
    return cat in CATEGORIES

def reachable_from(start: str, allowed_only: bool = True) -> set:
    """BFS to find all categories reachable from start."""
    visited = set()
    queue = [start]
    while queue:
        current = queue.pop(0)
        if current in visited:
            continue
        visited.add(current)
        for t in TRANSITIONS:
            if t["from"] == current and (not allowed_only or t["allowed"]):
                if t["to"] not in visited:
                    queue.append(t["to"])
    return visited

def path_to_authority(start: str) -> Optional[list]:
    """DFS to find a path from start to any authority-plane category."""
    if start in AUTHORITY_PLANE:
        return None  # Already in authority plane, not a custody→authority path
    visited = set()
    def dfs(current, path):
        if current in visited:
            return None
        visited.add(current)
        path = path + [current]
        if current in AUTHORITY_PLANE and len(path) > 1:
            return path
        for t in TRANSITIONS:
            if t["from"] == current and t["allowed"]:
                result = dfs(t["to"], path)
                if result:
                    return result
        return None
    return dfs(start, [])

def firewall_check(from_cat: str, to_cat: str) -> dict:
    """Run the firewall check on a proposed transition."""
    violations = []

    # Check 1: Known categories
    if not is_known_category(from_cat):
        violations.append(f"Unknown source category: {from_cat}")
    if not is_known_category(to_cat):
        violations.append(f"Unknown target category: {to_cat}")
    if violations:
        return {"violations": violations, "clean": False}

    # Check 2: Transition in matrix and allowed
    result = check_transition(from_cat, to_cat)
    if result is None:
        violations.append(f"Transition not in matrix: {from_cat} → {to_cat}")
    elif result is False:
        violations.append(f"Forbidden transition: {from_cat} → {to_cat}")

    # Check 3: No path to authority from custody-plane source
    if from_cat in CUSTODY_PLANE:
        auth_path = path_to_authority(from_cat)
        if auth_path:
            violations.append(f"Authority reachable from {from_cat}: path={auth_path}")

    return {"violations": violations, "clean": len(violations) == 0}

def reachability_check(category: str) -> dict:
    """Check if authority is reachable from a category."""
    path = path_to_authority(category)
    if path:
        return {"authority_reachable": True, "clean": False, "path": path}
    return {"authority_reachable": False, "clean": True, "path": None}

def validate_constraints(constraints: dict) -> list:
    """Validate that all constitutional constraints are false."""
    violations = []
    for key in ["authority_granted", "permission_granted", "jurisdiction_crossed"]:
        if constraints.get(key, False):
            violations.append(f"{key} is true (must be false)")
    return violations


# ── Receipt Generation ─────────────────────────────────────────────────────

HARNESS_IDENTITY = "tscp_acceptance_harness"

def generate_receipt(fco: dict) -> dict:
    """
    Generate a conformance receipt from a Formal Custody Object.

    This is an Evidence Generator — it produces a record that a declared
    procedure verified conformance. It does NOT grant authority.

    Returns a receipt dict. If the FCO is invalid, returns a rejection receipt.
    """
    from_cat = fco.get("category", "Custody")
    to_cat = "AcceptanceReceipt"

    # Run firewall check
    fw_result = firewall_check(from_cat, to_cat)

    # Check reachability
    reach_result = reachability_check(from_cat)

    # Validate constraints
    constraints = fco.get("constraints", {})
    constraint_violations = validate_constraints(constraints)

    # All violations
    all_violations = fw_result["violations"] + constraint_violations
    if not reach_result["clean"]:
        all_violations.append(f"Reachability failure: {reach_result}")

    # Build receipt
    content_for_hash = json.dumps({
        "produced_by": HARNESS_IDENTITY,
        "input_fco_hash": fco.get("content_hash", ""),
        "transition": {"from": from_cat, "to": to_cat, "allowed": fw_result["clean"]},
        "violations": all_violations,
    }, sort_keys=True)

    receipt_hash = hashlib.sha256(content_for_hash.encode()).hexdigest()
    receipt_id = receipt_hash[:16]

    receipt = {
        "receipt_id": receipt_id,
        "receipt_hash": receipt_hash,
        "produced_by": HARNESS_IDENTITY,
        "produced_at": datetime.now(timezone.utc).isoformat(),
        "input_fco_hash": fco.get("content_hash", ""),
        "transition_verified": {
            "from": from_cat,
            "to": to_cat,
            "allowed": fw_result["clean"],
        },
        "firewall_result": {
            "violations": fw_result["violations"],
            "clean": fw_result["clean"],
        },
        "reachability_result": {
            "authority_reachable": reach_result["authority_reachable"],
            "clean": reach_result["clean"],
        },
        "constitutional_constraints": {
            "authority_granted": False,
            "permission_granted": False,
            "jurisdiction_crossed": False,
        },
        "statement": "A declared procedure produced this conformance artifact.",
        "valid": len(all_violations) == 0,
        "violations": all_violations,
    }

    return receipt


# ── CLI ────────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    import sys
    if len(sys.argv) < 2:
        print("Usage: tscp_acceptance_harness.py <fco.json>")
        sys.exit(1)
    with open(sys.argv[1]) as f:
        fco = json.load(f)
    receipt = generate_receipt(fco)
    print(json.dumps(receipt, indent=2))
    sys.exit(0 if receipt["valid"] else 1)
