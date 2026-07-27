# FCO Receipt Composition Algebra Specification
## TSCP Custody Plane — Receipt Aggregation Without Authority Amplification

**Version:** 1.0
**Date:** 2026-07-26
**Status:** Sealed (evidence surface — not authority surface)
**Algebra target:** FCO_TRANSITION_ALGEBRA.md v1.1
**Commit:** pending

---

## 1. Purpose

The FCO Transition Algebra defines the transition topology of the custody plane. This document defines how receipts — the evidence artifacts produced by the custody plane — compose without creating authority.

The core problem: two valid receipts, combined, must not produce a receipt that is "more authoritative" than either component. Composition may increase evidence. It may not increase jurisdiction.

$$\boxed{\text{Composition may increase evidence, never jurisdiction}}$$

---

## 2. Receipt Type Definition

### 2.1 The Receipt Tuple

A receipt is a tuple:

$$r = (\text{id}, \text{subject}, \text{evidence}, \text{claim}, \text{lineage}, \text{timestamp}, \text{signature})$$

| Field | Type | Role |
|---|---|---|
| id | Hash | Content-addressed identity (SHA-256 of content) |
| subject | String | What the receipt is about (e.g., "NTT backend equivalence") |
| evidence | List | Artifacts supporting the claim (test logs, hashes, proofs) |
| claim | Claim | What is asserted (see §5) |
| lineage | DAG | Provenance chain — how this receipt came to exist |
| timestamp | ISO-8601 | When the receipt was produced |
| signature | Hash | Cryptographic commitment to the receipt content |

### 2.2 The Missing Field

The receipt type does **not** contain:

- `authority`
- `permission`
- `jurisdiction`
- `execution_rights`
- `control`

This omission is a **type-level boundary**, not a design choice. A receipt that structurally cannot contain authority cannot accidentally become an authority-bearing object without a category transition that the firewall can reject.

### 2.3 Negative Receipts

A negative receipt is a receipt where the claim is a failure:

$$r_{drift} = (\text{id}, \text{subject}, \text{evidence}, \text{claim}_{fail}, \text{lineage}, \text{timestamp}, \text{signature})$$

where $\text{claim}_{fail}$ asserts that a property was NOT satisfied under a specific test.

Negative receipts are first-class custody data. A mature custody system preserves both:

- **Positive evidence:** "I satisfy property P"
- **Negative evidence:** "I failed property P under test T"

Both are lineage. Neither is authority.

---

## 3. Authority Exclusion Theorem

### 3.1 Statement

$$\forall r \in \mathcal{R}: \quad \text{Authority}(r) = \bot$$

No receipt contains authority. This is structural, not procedural.

### 3.2 Proof

1. The receipt type $\mathcal{R}$ is defined as the product of seven fields (§2.1), none of which is authority.
2. The only constructor for $\mathcal{R}$ produces a tuple of exactly those seven fields.
3. No field in the tuple has type "authority" or any subtype that could carry authority.
4. Therefore, no receipt can contain authority. $\blacksquare$

### 3.3 Composition Preservation

$$\forall r_1, r_2 \in \mathcal{R}: \quad \text{Authority}(r_1 \oplus r_2) = \bot$$

**Proof:** The composition operator $\oplus$ (§4) produces a receipt $r_3$ whose type is $\mathcal{R}$. By §3.2, $\text{Authority}(r_3) = \bot$. The composition operator does not introduce new fields. $\blacksquare$

In type-system terms:

```
Receipt
 ├── Evidence
 ├── Claim
 ├── Lineage
 └── Signature

(no Authority branch exists)
```

The invariant is structural: the type excludes authority, and the operator preserves the type.

---

## 4. Composition Operator

### 4.1 Definition

The composition operator is a **partial** function:

$$\oplus: \mathcal{R} \times \mathcal{R} \rightharpoonup \mathcal{R}$$

Partiality is essential. An unrestricted merge would create the dangerous assumption: "Two valid things combined must produce a valid larger thing." That is exactly the type of implicit authority escalation the custody plane prevents.

### 4.2 Result Fields

When $r_1 \oplus r_2 = r_3$ is defined:

| Field | Value |
|---|---|
| id | SHA-256 of content |
| subject | $r_1.\text{subject}$ (must equal $r_2.\text{subject}$) |
| evidence | $r_1.\text{evidence} \cup r_2.\text{evidence}$ (union) |
| claim | $r_1.\text{claim} \wedge r_2.\text{claim}$ (conjunction, see §5) |
| lineage | $r_1.\text{lineage} \oplus_{DAG} r_2.\text{lineage}$ (DAG union) |
| timestamp | $\max(r_1.\text{timestamp}, r_2.\text{timestamp})$ |
| signature | SHA-256 of composed content |

### 4.3 Undefined Cases

The operator is undefined (returns $\bot$) when any guard fails (§5).

---

## 5. Partiality Rules (Composition Guards)

### 5.1 Subject Agreement

$$r_1.\text{subject} = r_2.\text{subject}$$

Receipts about different subjects cannot compose. This prevents unrelated evidence aggregation — combining evidence about NTT correctness with evidence about Montgomery arithmetic does not produce a receipt about either.

### 5.2 Claim Consistency

$$\neg(r_1.\text{claim} \text{ contradicts } r_2.\text{claim})$$

Claims must not contradict. A receipt asserting "backend A matches reference" cannot compose with a receipt asserting "backend A does not match reference."

### 5.3 Lineage Acyclicity

$$\text{lineage}(r_1) \oplus_{DAG} \text{lineage}(r_2) \text{ is acyclic}$$

Composition must not create cycles in the lineage graph. This prevents circular self-justification: $r_1$ depends on $r_2$ which depends on $r_1$.

### 5.4 Evidence Independence

$$\text{evidence}(r_1) \cap \text{evidence}(r_2) = \emptyset \text{ or shared evidence is explicitly declared}$$

Evidence sources must be independent. If two receipts share an evidence artifact, that sharing must be explicit in the lineage, not implicit through composition. This prevents hidden self-attestation: the same test log used to support two different claims would appear to be independent evidence if the sharing were not declared.

---

## 6. Claim Dependency Graph

### 6.1 The Refined Claim Model

A claim is a triple:

$$\text{claim} = (\text{statement}, \text{dependencies}, \text{scope})$$

| Component | Type | Role |
|---|---|---|
| statement | String | What is asserted |
| dependencies | List | What evidence supports the statement |
| scope | String | Where the claim applies |

### 6.2 Why This Matters

Conjunction alone does not mean strengthening. Consider:

**Receipt A:** "AVX backend matches scalar backend" (scope: arithmetic correctness)
**Receipt B:** "Scalar backend matches DFT" (scope: arithmetic correctness)

The composite claim "AVX matches scalar AND scalar matches DFT" is reasonable. But it does not mean every property of the AVX backend is now proven. The scope limits what the composition establishes.

Without the dependency graph, composition would treat conjunction as automatic strengthening. With it, the composition rule becomes:

$$C_{combined} = C_1 \wedge C_2 \quad \text{only if dependency graphs remain valid}$$

### 6.3 Composition Rule

When $r_1 \oplus r_2 = r_3$:

$$r_3.\text{claim} = (s_1 \wedge s_2, \; d_1 \cup d_2, \; \text{scope}_1 \cap \text{scope}_2)$$

The scope **intersects** (narrows), not unions (widens). The composed claim applies only where both component claims apply.

---

## 7. Lineage DAG Constraints

### 7.1 Lineage as a DAG

Each receipt's lineage is a directed acyclic graph where:
- Nodes are receipt IDs
- Edges represent "depends on" relationships
- The root is the current receipt

### 7.2 Composition Extends the DAG

When $r_1 \oplus r_2 = r_3$:
- $r_3$'s lineage includes all nodes and edges from $r_1$ and $r_2$
- $r_3$'s lineage adds edges: $r_3 \to r_1$ and $r_3 \to r_2$
- The result must remain acyclic

### 7.3 Cycle Detection

A cycle would mean: $r_a$ depends on $r_b$ which depends on ... which depends on $r_a$. This is circular self-justification — the same failure mode as the prohibited loop in the transition algebra (§5.3 of FCO_TRANSITION_ALGEBRA.md).

---

## 8. Drift Classification

### 8.1 Semantic Drift vs Invalid Implementation

Semantic drift is not always an implementation defect. The custody model preserves the ability to locate which layer moved:

| Case | Classification | Meaning |
|---|---|---|
| Code wrong relative to spec | Implementation Drift | The implementation diverged |
| Spec ambiguous | Specification Defect | The specification was unclear |
| Reference wrong | Oracle Defect | The semantic oracle was wrong |
| Both changed independently | Lineage Divergence | Both moved; neither is wrong alone |

### 8.2 Drift Receipt Format

A drift receipt includes:

| Field | Content |
|---|---|
| subject | What was tested |
| claim.statement | "Property P failed under test T" |
| claim.dependencies | Oracle identity, test vector identity |
| claim.scope | What the failure applies to |
| evidence | Test output, oracle output, deviation measurement |
| lineage | Oracle custody chain + implementation custody chain |

### 8.3 Negative Receipt Composition

Negative receipts participate in composition under the same rules:
- Subject agreement: both receipts must address the same subject
- Claim consistency: a negative receipt and a positive receipt about the same property under the same test are contradictory (composition undefined)
- A negative receipt about property P under test T₁ and a positive receipt about property P under test T₂ may compose (different tests, different scope)

---

## 9. Mapping to Implementations

### 9.1 JSON Model (`receipt_composition.json`)

The JSON model defines:
- Receipt type schema (7 fields, no authority)
- Claim structure (statement, dependencies, scope)
- Composition rules (guards and result field computation)
- Drift receipt extension

### 9.2 Rust Validator (`receipt_validator.rs`)

The Rust validator implements:
- `validate_receipt(r)` — checks receipt is well-formed (no authority field)
- `validate_composition(r1, r2)` — checks all four guards
- `compose(r1, r2)` — performs composition if guards pass, returns None otherwise
- `check_authority_exclusion(r)` — verifies no authority field exists

### 9.3 Lean Model (`ReceiptComposition.lean`)

The Lean model mechanizes:
- Receipt inductive type (no Authority constructor)
- `authority_exclusion`: ∀ r, Authority(r) = false (by structural inspection)
- `composition_preserves_authority_exclusion`: ∀ r₁ r₂, Authority(r₁ ⊕ r₂) = false
- Drift classification inductive type

### 9.4 External Verifier (`composition_verifier.py`)

The external verifier independently checks:
- Receipt type completeness (exactly 7 fields, no authority)
- Composition guard correctness (all 4 guards enforced)
- Authority exclusion (structural, not procedural)
- Negative receipt handling (first-class, not special-cased)
- Drift classification (4 classes, diagnostically distinct)

---

## 10. Combined Invariant

The receipt composition algebra preserves the core architectural invariant:

$$\boxed{\text{Evidence can become richer. Authority cannot emerge.}}$$

Formally:

$$\forall r \in \mathcal{R}: \text{Authority}(r) = \bot$$

$$\forall r_1, r_2 \in \mathcal{R}: \text{Authority}(r_1 \oplus r_2) = \bot$$

$$\forall r_1, r_2 \in \mathcal{R}: r_1 \oplus r_2 = \bot \text{ (undefined) if any guard fails}$$

The receipt algebra closes the aggregation loophole. Combined with the transition algebra (which closes the boundary-crossing loophole) and the semantic reference protocol (which closes the meaning loophole), the custody plane now has three layers of structural protection:

1. **Transition Algebra** — no path from custody to authority
2. **Receipt Composition** — no authority field in the receipt type
3. **Semantic Reference** — drift detection independent of internal consistency

---

## 11. Change Protocol

Same as FCO Transition Algebra §10. Any modification requires:
1. Updated JSON model
2. Updated Rust validator
3. Updated Lean model (re-proven)
4. Updated external verifier
5. CI gate passes
6. New commit with evidence of all changes

The receipt type is frozen. Adding an authority field is an architectural change requiring full re-verification.
