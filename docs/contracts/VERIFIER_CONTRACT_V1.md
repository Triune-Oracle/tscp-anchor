VERIFIER_CONTRACT_V1.md

Purpose

Verifier Contract V1 defines the stable public interface of the TSCP verifier.

This contract governs the observable behavior of:

tscp verify <bundle>

The verifier implementation, proving backend, cryptographic primitives, internal crate structure, and operational infrastructure are explicitly outside the scope of this contract.

---

Output Contract

The verifier MUST emit exactly one of the following results.

Success

PASS

Failure

FAIL: malformed_bundle
FAIL: wrong_version
FAIL: modified_receipt
FAIL: wrong_parent
FAIL: forged_signature
FAIL: internal_error

No additional output is part of the public verifier contract.

The verifier contract does not guarantee:

- stack traces
- timestamps
- file paths
- crate names
- implementation details
- cryptographic details
- diagnostic explanations

Such information MAY be logged internally but MUST NOT be relied upon by external consumers.

---

Exit Codes

Exit Code| Result
0| PASS
1| modified_receipt
2| wrong_parent
3| forged_signature
4| wrong_version
5| malformed_bundle
255| internal_error

Exit-code meanings are part of Verifier Contract V1 and are considered stable.

---

Compatibility Guarantee

All output strings and exit-code meanings defined by Verifier Contract V1 are frozen for the lifetime of Verifier Contract V1.

New implementation-level failures MUST map to an existing Verifier Contract V1 result code or to:

FAIL: internal_error

Adding a new public result code requires a new verifier contract version.

Consumers SHOULD rely only on:

- the exact output strings defined above
- the exit-code meanings defined above

Consumers MUST NOT rely on implementation details, diagnostics, logs, or undocumented behavior.

---

Scope

Verifier Contract V1 defines only the public verification surface:

Input Bundle
    ↓
tscp verify
    ↓
PASS | FAIL:<code>

Everything else is implementation.
