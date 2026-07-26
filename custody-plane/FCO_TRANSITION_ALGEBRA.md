# FCO Transition Algebra Specification
## TSCP Custody Plane — Canonical Formal Document

**Version:** 1.0
**Date:** 2026-07-26
**Status:** Sealed (evidence surface — not authority surface)
**Commit:** 579cc02

---

## 1. Purpose

This document is the human-readable bridge between three machine representations of the custody plane:

| Representation | File | Role |
|---|---|---|
| Transition matrix | `transition_matrix.json` | Declarative topology |
| Structural firewall | `boundary_firewall.rs` | Executable boundary check |
| Formal invariants | `FCO_Invariants.lean` | Mechanized proof |

All three implement the same algebra. This document defines it.

A reader who understands this document can verify all three implementations by hand.

---

## 2. Definitions

### 2.1 Categories

The custody plane has five categories:

$$\mathcal{C}_{full} = \{ \text{Custody}, \text{Evidence}, \text{AcceptanceReceipt}, \text{Authority}, \text{Execution} \}$$

### 2.2 Planes

Categories are partitioned into two planes:

$$\text{Custody Plane:} \quad C = \{ \text{Custody}, \text{Evidence}, \text{AcceptanceReceipt} \}$$

$$\text{Authority Plane:} \quad A = \{ \text{Authority}, \text{Execution} \}$$

**Axiom (Plane Partition):**

$$C \cap A = \emptyset, \quad C \cup A = \mathcal{C}_{full}$$

Every category belongs to exactly one plane. No category is in both.

### 2.3 Transition Relation

The transition relation `allowed` is a total function:

$$\text{allowed}: \mathcal{C}_{full} \times \mathcal{C}_{full} \to \{true, false\}$$

defined by the 5×5 transition matrix (25 entries, 10 allowed, 15 forbidden).

**Notation:** We write $a \to b$ when $\text{allowed}(a, b) = true$.

### 2.4 Reachability

Reachability is the reflexive-transitive closure of the transition relation:

$$\text{Reachable}(a, b) \iff \exists\; \text{path}\; a = c_0 \to c_1 \to \cdots \to c_n = b$$

for some $n \geq 0$, where each $c_i \to c_{i+1}$ is an allowed transition.

**Notation:** We write $a \leadsto b$ when $\text{Reachable}(a, b)$.

---

## 3. The Transition Matrix

The complete transition relation, enumerated:

### 3.1 Custody-Plane Internal Transitions (6 allowed)

| From | To | Allowed | Meaning |
|---|---|---|---|
| Custody | Custody | ✓ | Custody may self-reference |
| Custody | Evidence | ✓ | Custody may produce evidence |
| Custody | AcceptanceReceipt | ✓ | Custody may receive acceptance |
| Evidence | Evidence | ✓ | Evidence may self-reference |
| Evidence | AcceptanceReceipt | ✓ | Evidence may receive acceptance |
| AcceptanceReceipt | AcceptanceReceipt | ✓ | Receipts may self-reference |

| From | To | Allowed | Meaning |
|---|---|---|---|
| Evidence | Custody | ✗ | Evidence may not assert custody |
| AcceptanceReceipt | Custody | ✗ | Receipt may not assert custody |
| AcceptanceReceipt | Evidence | ✓ | Receipt may produce evidence |

### 3.2 Custody → Authority Transitions (6 forbidden)

| From | To | Allowed | Reason |
|---|---|---|---|
| Custody | Authority | ✗ | Representation may not create jurisdiction |
| Custody | Execution | ✗ | Representation may not create execution |
| Evidence | Authority | ✗ | Evidence may not create authority |
| Evidence | Execution | ✗ | Evidence may not create execution |
| AcceptanceReceipt | Authority | ✗ | Receipt may not create authority |
| AcceptanceReceipt | Execution | ✗ | Receipt may not create execution |

**These are the critical invariants.** No transition from the custody plane to the authority plane is allowed.

### 3.3 Authority-Plane Internal Transitions (2 allowed)

| From | To | Allowed | Meaning |
|---|---|---|---|
| Authority | Authority | ✓ | Authority may self-reference |
| Authority | Execution | ✓ | Authority may grant execution |

| From | To | Allowed | Reason |
|---|---|---|---|
| Authority | Custody | ✗ | Authority may not assert custody |
| Authority | Evidence | ✗ | Authority may not fabricate evidence |
| Authority | AcceptanceReceipt | ✗ | Authority may not self-receipt |

### 3.4 Authority → Custody Transitions (2 allowed, 3 forbidden)

| From | To | Allowed | Meaning |
|---|---|---|---|
| Execution | Execution | ✓ | Execution may self-reference |
| Execution | Evidence | ✓ | **Execution may produce evidence** |

| From | To | Allowed | Reason |
|---|---|---|---|
| Execution | Custody | ✗ | Execution may not assert custody |
| Execution | Authority | ✗ | Execution may not self-authorize |
| Execution | AcceptanceReceipt | ✗ | Execution may not self-receipt |

---

## 4. The Key Invariant

### 4.1 Statement

$$\boxed{\forall c \in C, \quad \neg\;\text{Reachable}(c, \text{Authority})}$$

No category in the custody plane can reach any category in the authority plane through a sequence of allowed transitions.

### 4.2 Proof (by enumeration)

The proof is by exhaustive enumeration of the finite state space (5 categories, 25 transitions). It is mechanized in Lean as `custody_receipt_no_authority_path`, proven by `decide`.

**Manual verification:**

From **Custody**, allowed transitions go to: {Custody, Evidence, AcceptanceReceipt} — all in $C$.

From **Evidence**, allowed transitions go to: {Evidence, AcceptanceReceipt} — all in $C$.

From **AcceptanceReceipt**, allowed transitions go to: {AcceptanceReceipt, Evidence} — all in $C$.

By induction on path length, every category reachable from any $c \in C$ is in $C$. Since $\text{Authority} \in A$ and $C \cap A = \emptyset$, Authority is unreachable. $\blacksquare$

### 4.3 Strengthened Form

$$\forall c \in C, \forall a \in A, \quad \neg\;\text{Reachable}(c, a)$$

No custody-plane category can reach any authority-plane category.

This is mechanized as `custody_plane_separation` in Lean.

---

## 5. The Directionality Principle

### 5.1 The Asymmetry

The transition relation is **not symmetric**. This is by design.

**Authority may produce evidence:**

$$\text{Execution} \to \text{Evidence} \quad \text{(allowed)}$$

Execution naturally produces evidence about what it did. This is a one-way flow: the authority plane emits evidence into the custody plane. The evidence records what happened; it does not grant authority back to the executor.

**Evidence may not produce authority:**

$$\text{Evidence} \to \text{Authority} \quad \text{(forbidden)}$$

Evidence is a representation of what occurred. A representation cannot create jurisdiction. If evidence could produce authority, then any artifact that records an event would inherit the authority of that event — collapsing the boundary.

### 5.2 The Directionality Axiom

$$\boxed{\text{Authority may generate evidence, but evidence may not generate authority.}}$$

Formally:

$$\exists\; a \in A, c \in C: \quad a \to c \quad \text{(permitted)}$$

$$\neg\exists\; c \in C, a \in A: \quad c \to a \quad \text{(forbidden)}$$

The transition relation is **upper-triangular with respect to the plane ordering** $C < A$: transitions may flow downward (authority → custody) but not upward (custody → authority).

### 5.3 The Prohibited Loop

The forbidden pattern is:

$$\text{Custody} \leadsto \text{Authority} \leadsto \text{Custody}$$

This would allow a custody object to:
1. Produce an authority grant (custody → authority)
2. Receive evidence from that grant (authority → custody)
3. Use the evidence to justify the original grant

This is the **self-attestation** failure mode: the verifier and the verified are the same entity. The transition algebra prevents it by blocking step 1.

### 5.4 The Permitted Flow

The allowed pattern is:

$$\text{Authority} \to \text{Execution} \to \text{Evidence} \to \text{AcceptanceReceipt}$$

Authority grants execution. Execution produces evidence. Evidence is recorded in a receipt. The receipt is in the custody plane. At no point does the receipt gain the ability to produce authority.

---

## 6. Transition Algebra

### 6.1 Composition

Transitions compose by path concatenation:

$$\text{If } a \to b \text{ and } b \to c, \text{ then } a \leadsto c$$

Reachability is the closure under composition.

### 6.2 Plane Closure

**Theorem (Custody-Plane Closure):**

$$\forall c \in C, \quad \text{Reachable}(c) \subseteq C$$

The set of categories reachable from any custody-plane category is a subset of the custody plane.

**Proof:** By the key invariant (§4), no $a \in A$ is reachable from any $c \in C$. Since $C \cup A = \mathcal{C}_{full}$, every reachable category is in $C$. $\blacksquare$

### 6.3 Monotonicity

**Theorem (Evidence Monotonicity):**

Adding evidence never creates a path to authority.

$$\text{If } \neg\;\text{Reachable}(c, a) \text{ for all } c \in C, a \in A, \text{ then adding any } c' \to c'' \text{ where } c', c'' \in C \text{ preserves this.}$$

Adding a custody-internal transition cannot create a custody-to-authority path because no custody-internal transition connects to the authority plane.

**Corollary:** The custody plane is closed under evidence production. Producing more evidence (adding transitions within $C$) does not create authority.

$$\boxed{\text{More evidence} \neq \text{More authority}}$$

### 6.4 Non-Monotonicity at the Boundary

Adding a transition $c \to a$ for $c \in C, a \in A$ **does** create a path to authority. The firewall rejects such additions.

This is why the CI gate checks for matrix drift: any change to the transition matrix that adds a custody-to-authority edge violates the invariant.

### 6.5 Duality

The two inverse directions serve different roles:

| Direction | Role | Permitted | Why |
|---|---|---|---|
| Authority → Evidence | Evidence production | ✓ | Execution records what it did |
| Evidence → Authority | Authority creation | ✗ | Representation cannot create jurisdiction |

The duality is asymmetric. The custody plane receives from the authority plane but does not transmit to it. This is the structural expression of:

$$\text{AcceptanceReceipt} = \text{Evidence of Conformance}$$

$$\text{AcceptanceReceipt} \neq \text{Permission}$$

---

## 7. Mapping to Implementations

### 7.1 JSON Matrix (`transition_matrix.json`)

The JSON matrix is the declarative form of §3. Each entry corresponds to one cell in the 5×5 grid. The `allowed` boolean is the transition relation.

### 7.2 Rust Firewall (`boundary_firewall.rs`)

The Rust firewall implements §4 and §5:
- `check_transition(a, b)` implements §3 (lookup in the matrix)
- `path_to_authority(start)` implements §4 (reachability check)
- `validate_constraints(fco)` implements §5.2 (constraints are const:false)
- `Firewall::check(from, to)` composes all checks

The firewall is a **topology gate**: it accepts allowed transitions and rejects forbidden ones. It does not evaluate semantics, compute trust, or infer governance.

### 7.3 Lean Invariants (`FCO_Invariants.lean`)

The Lean model mechanizes §4 and §6:
- `canReach` is the transitive closure (§2.4), computed by pattern matching over all 25 pairs
- `custody_receipt_no_authority_path` is §4.1 (key invariant)
- `custody_plane_separation` is §4.3 (strengthened form)
- `no_plane_crossing` is §5.2 (directionality axiom)
- `harness_is_evidence_generator` is §6.5 (receipt ≠ permission)

All theorems are proven by `decide` — exhaustive enumeration of the finite state space. No axioms, no `sorry`.

### 7.4 External Verifier (`external_verifier.py`)

The external verifier re-derives §3, §4, and §5 from the JSON matrix without trusting the Lean proof or the Rust firewall:
- `verify_matrix_completeness` checks §3 (25/25 entries)
- `verify_plane_separation` checks §5.2 (no custody→authority crossing)
- `verify_reachability` checks §4 (BFS from each custody category)
- `verify_constraint_schema` checks §6.5 (constraints are const:false)
- `verify_firewall_consistency` checks §5.1 (custody→authority forbidden, authority→custody permitted)

---

## 8. The Custody Chain

The transition algebra is the formal backbone of the custody chain:

```
Constitution (this document)
    |
    v
Transition Topology (§3 — the matrix)
    |
    v
Custody Boundary (§4 — the key invariant)
    |
    v
Structural Firewall (§7.2 — Rust)
    |
    v
Formal Conformance (§7.3 — Lean)
    |
    v
Acceptance Receipt (§6.5 — evidence, not permission)
    |
    v
Custody Lineage (§7.4 — external verification)
```

Each layer implements the layer above it. The algebra is the specification; the implementations are the evidence that the specification is enforced.

The chain terminates at **custody evidence**, not jurisdiction.

---

## 9. Classification

This document is an **Evidence Artifact**, not an Authority Artifact.

It specifies what the custody plane **is**. It does not specify what the custody plane **should control**.

It defines the rules of the boundary. It does not grant permission to cross it.

$$\boxed{\textbf{Specification} \neq \textbf{Jurisdiction}}$$

---

## 10. Change Protocol

Any modification to this document requires:

1. Updated transition matrix (JSON)
2. Updated firewall (Rust)
3. Updated invariants (Lean, re-proven)
4. Updated external verifier (Python)
5. CI gate passes (all 6 jobs)
6. New commit with evidence of all changes

The algebra is frozen. Changes to the invariant (§4) are architectural changes, not implementation changes, and require the full custody chain to be re-verified.
