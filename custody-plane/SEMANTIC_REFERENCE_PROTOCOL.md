# Semantic Reference Protocol Specification
## TSCP Custody Plane — Drift Detection Independent of Internal Consistency

**Version:** 1.0
**Date:** 2026-07-26
**Status:** Sealed (evidence surface — not authority surface)
**Algebra target:** FCO_TRANSITION_ALGEBRA.md v1.1
**Commit:** pending

---

## 1. Purpose

The FCO Transition Algebra defines the topology of the custody plane. The Receipt Composition Algebra defines how evidence aggregates without authority amplification. This document defines how semantic correctness is verified independently of internal consistency.

The core problem: an implementation can be internally consistent (all backends agree with each other) yet semantically wrong (all backends implement the wrong algorithm). This is **semantic drift** — a deeper failure class than ordinary inconsistency.

$$\text{Drift}(m) \equiv \text{Impl}(m) \not\equiv_{\text{sem}} \text{Ref}(m)$$

$$\text{Consistent}(\text{Impl}(m)) = \top \quad \text{(drift is possible even when internally consistent)}$$

---

## 2. Semantic Drift vs Ordinary Inconsistency

### 2.1 Ordinary Inconsistency

$$\text{backend}_A \neq \text{backend}_B$$

One implementation disagrees with another. This is a bug. It is detectable by cross-backend comparison.

### 2.2 Semantic Drift

$$\text{backend}_A = \text{backend}_B \quad \text{but} \quad \text{backend}_A \neq \text{mathematical object}$$

All implementations agree with each other but disagree with the mathematical definition. This is drift. It is NOT detectable by cross-backend comparison alone — it requires an external semantic oracle.

### 2.3 The DIF Incident (Historical Reference)

During TSCP development, a DIT butterfly was used inside a DIF NTT structure. All backends agreed (internal consistency passed). But the mathematical result was wrong (the output was a DIT transform, not a DIF transform). This was semantic drift, not inconsistency.

The failure was detected only because reference vectors (independent test data from the mathematical definition of DIF) were compared against the implementation output.

---

## 3. Oracle Independence Requirements

### 3.1 Lineage Separation

The semantic oracle **cannot** descend from the implementation being checked. Otherwise:

$$\text{Impl} \rightarrow \text{Ref} \rightarrow \text{Impl}$$

creates circular evidence. The oracle would verify the implementation against a reference derived from the implementation itself — the self-attestation failure mode.

**Requirement:** The oracle's custody chain must be independent from the implementation's custody chain. They may share the mathematical definition as a common ancestor, but the oracle must not contain any implementation artifact in its lineage.

### 3.2 Durability

The semantic reference requires its own custody chain:

```
Mathematical Definition
        |
        v
Reference Vectors
        |
        v
Independent Oracle
        |
        v
Content Commitment (hash)
```

Each stage is a custody artifact. The oracle itself becomes a custody artifact, not an authority artifact. It records "this reference was derived from this mathematical definition" — it does not assert "this reference is authoritative."

**Requirement:** The oracle must have a content commitment (hash) that can be independently verified against the mathematical definition.

### 3.3 Coverage

This is the hardest requirement. The reference must distinguish **neighboring** mathematical objects — not just validate correct behavior.

For the DIF incident, insufficient coverage was:

> "Forward transform produces plausible output"

Required coverage was:

> "Declared DIF transform produces output matching DIF reference vectors, AND differs from DIT transform output, AND differs from inverse transform output, AND differs from alternative ordering output"

**Requirement:** Test vectors must target semantic boundaries — the points where neighboring mathematical objects produce different outputs. Testing only normal operation misses drift because drift produces plausible-but-wrong outputs.

### 3.4 Coverage Taxonomy

| Coverage Type | Detects | Misses |
|---|---|---|
| Normal operation | Crashes, obvious errors | Semantic drift |
| Boundary cases | Edge behavior | Near-miss algorithms |
| Semantic boundaries | Neighboring algorithm confusion | Nothing in this class |
| Cross-structure | Internal inconsistency | Correctness vs definition |

A mature custody system requires all four coverage types. The semantic boundary coverage is the one that catches drift.

---

## 4. Drift Classification

### 4.1 Four Classes

The custody model preserves the ability to locate which layer moved:

| Case | Classification | Meaning | Action |
|---|---|---|---|
| Code wrong relative to spec | Implementation Drift | The implementation diverged from the specification | Fix the implementation |
| Spec ambiguous | Specification Defect | The specification was unclear enough to allow multiple readings | Fix the specification |
| Reference wrong | Oracle Defect | The semantic oracle (reference vectors) was wrong | Fix the oracle, re-verify all implementations |
| Both changed independently | Lineage Divergence | Both implementation and spec moved; neither is wrong alone | Reconcile or branch |

### 4.2 Why Classification Matters

Without classification, every drift event looks like "the implementation is wrong." But:

- **Implementation Drift** → fix the code
- **Specification Defect** → fix the spec, re-verify all implementations (they may have been "correct" relative to the old spec)
- **Oracle Defect** → fix the oracle, re-verify ALL implementations (the oracle may have been validating wrong behavior)
- **Lineage Divergence** → this is the hardest case: both the spec and implementation evolved, and the divergence is not a bug in either

Treating all drift as implementation drift would cause:
1. Implementations to be "fixed" to match a wrong oracle (Oracle Defect)
2. Specs to be ignored when they conflict with code (Specification Defect)
3. Legitimate parallel evolution to be treated as a bug (Lineage Divergence)

### 4.3 Classification Protocol

When drift is detected:

1. Compare implementation output to oracle output → deviation exists
2. Check oracle against mathematical definition → is the oracle correct?
3. Check specification against mathematical definition → is the spec clear?
4. Check implementation against specification → is the code following the spec?
5. Classify based on which checks fail:
   - Oracle wrong → Oracle Defect
   - Spec ambiguous → Specification Defect
   - Code wrong, oracle right, spec clear → Implementation Drift
   - Both changed → Lineage Divergence

---

## 5. Drift Receipt Format

### 5.1 Negative Receipt

A drift detection event produces a negative receipt:

$$r_{drift} = (\text{id}, \text{subject}, \text{evidence}, \text{claim}_{fail}, \text{lineage}, \text{timestamp}, \text{signature})$$

### 5.2 Fields

| Field | Content |
|---|---|
| id | SHA-256 of content |
| subject | "Semantic drift detected: {module}" |
| evidence | Implementation output, oracle output, deviation vectors |
| claim.statement | "Property P failed under test T using oracle O" |
| claim.dependencies | [oracle_identity, test_vector_identity] |
| claim.scope | Affected scope (e.g., "DIF NTT at sizes 16-4096") |
| lineage | Oracle custody chain + implementation custody chain (kept separate) |
| timestamp | Detection time |
| signature | Content hash |

### 5.3 Classification in the Receipt

The receipt includes a `classification` field in the evidence:

```json
{
  "classification": "ImplementationDrift",
  "oracle_identity": "sha256:...",
  "oracle_lineage_independent": true,
  "test_vector_coverage": "semantic_boundary",
  "deviation_magnitude": 0.001,
  "affected_scope": "DIF NTT sizes 16-4096"
}
```

The classification is evidence, not authority. It records what was found. It does not prescribe what to do about it.

---

## 6. Integration with Receipt Composition Algebra

### 6.1 Drift Receipts Participate in Composition

Negative receipts are first-class custody data and participate in the composition algebra:

- A drift receipt about module M under test T₁ can compose with a positive receipt about module M under test T₂ (different tests, different scope)
- A drift receipt and a positive receipt about the same property under the same test are **contradictory** — composition is undefined (claim consistency guard fails)
- Multiple drift receipts about the same module can compose (evidence union)

### 6.2 Composition Does Not Resolve Drift

Composition can aggregate evidence about drift, but it cannot resolve the drift. Resolving drift requires:

1. Classification (§4.3) to locate the layer that moved
2. Correction at the identified layer
3. Re-verification with an independent oracle
4. New positive receipt (if corrected) or updated negative receipt (if classified as spec/oracle defect)

The custody system preserves the evidence of drift. It does not prescribe the resolution. That is a human decision, not a custody operation.

---

## 7. The Extended Custody Chain

The semantic reference protocol extends the five-stage custody chain:

```
Specification
      ↓
Identity Binding
      ↓
Integrity Verification
      ↓
Conformance Evidence
      ↓
Semantic Agreement        ← NEW (this protocol)
      ↓
External Reproduction
```

### 7.1 What Each Stage Adds

| Stage | Without Semantic Protocol | With Semantic Protocol |
|---|---|---|
| Conformance | Implementation passes tests | Implementation passes tests AND matches oracle |
| Semantic Agreement | (not checked) | Implementation matches mathematical definition, not just internal consistency |
| External Reproduction | Independent verifier reproduces results | Independent verifier reproduces results AND can check against oracle |

### 7.2 The Strengthened Invariant

Without semantic verification:

$$\text{Consistent}(\text{Impl}) = \top \Rightarrow \text{Correct}(\text{Impl}) \quad \text{(FALSE — drift is possible)}$$

With semantic verification:

$$\text{Consistent}(\text{Impl}) = \top \wedge \text{Matches}(\text{Impl}, \text{Oracle}) = \top \Rightarrow \text{Correct}(\text{Impl}) \quad \text{(stronger)}$$

But even this is not authority:

$$\text{Correct}(\text{Impl}) \neq \text{Authority}(\text{Impl})$$

Correctness is evidence. Authority is jurisdiction. The custody plane terminates at evidence.

---

## 8. Classification

This document is an **Evidence Artifact**, not an Authority Artifact.

It defines how semantic drift is detected. It does not grant authority to resolve drift.

$$\boxed{\textbf{Detection} \neq \textbf{Resolution}}$$

$$\boxed{\textbf{Classification} \neq \textbf{Jurisdiction}}$$

---

## 9. Change Protocol

Same as FCO Transition Algebra §10. The oracle independence requirements (§3) are frozen — relaxing them is an architectural change requiring full re-verification.
