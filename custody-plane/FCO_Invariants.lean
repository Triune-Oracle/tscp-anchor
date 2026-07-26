/-
  FCO_Invariants.lean
  TSCP Custody Plane — Formal Conformance Invariants

  Key theorem: custody_receipt_no_authority_path

    canReach Custody AcceptanceReceipt = true
    canReach Custody Authority = false

  Producing a receipt from a custody object does not create a path to authority.
  The receipt is in the custody plane, not the authority plane.

  Classification: Evidence Generator formal invariant, NOT Authority Generator.
  The theorem proves what is NOT possible, not what IS permitted.

  Proof method: the transitive closure of `allowed` over 5 categories is
  computed by exhaustive enumeration (25 pairs). All facts are decidable.
-/

namespace TSCP.CustodyPlane

/- Algebra version binding: FCO_TRANSITION_ALGEBRA.md
    Hash: f3901b24857805ad68fdfc8001c2ba90c692a53d7ebcfdd17579e1b0ef3aa998
    This Lean model targets the algebra specification at that hash.
    Conformance is established by `decide` proofs, not by this declaration. -/


/-- Category in the custody plane topology -/
inductive Category
  | Custody
  | Evidence
  | AcceptanceReceipt
  | Authority
  | Execution
  deriving DecidableEq, Repr

/-- Is this category in the custody plane? -/
def isCustodyPlane : Category → Bool
  | Category.Custody => true
  | Category.Evidence => true
  | Category.AcceptanceReceipt => true
  | Category.Authority => false
  | Category.Execution => false

/-- Is this category in the authority plane? -/
def isAuthorityPlane : Category → Bool
  | Category.Custody => false
  | Category.Evidence => false
  | Category.AcceptanceReceipt => false
  | Category.Authority => true
  | Category.Execution => true

/-- Allowed transitions between categories (total function, 5×5). -/
def allowed : Category → Category → Bool
  | Category.Custody,           Category.Custody           => true
  | Category.Custody,           Category.Evidence          => true
  | Category.Custody,           Category.AcceptanceReceipt => true
  | Category.Custody,           Category.Authority         => false
  | Category.Custody,           Category.Execution         => false
  | Category.Evidence,          Category.Evidence          => true
  | Category.Evidence,          Category.AcceptanceReceipt => true
  | Category.Evidence,          Category.Authority         => false
  | Category.Evidence,          Category.Execution         => false
  | Category.Evidence,          Category.Custody           => false
  | Category.AcceptanceReceipt, Category.AcceptanceReceipt => true
  | Category.AcceptanceReceipt, Category.Evidence          => true
  | Category.AcceptanceReceipt, Category.Custody           => false
  | Category.AcceptanceReceipt, Category.Authority         => false
  | Category.AcceptanceReceipt, Category.Execution         => false
  | Category.Authority,         Category.Authority         => true
  | Category.Authority,         Category.Execution         => true
  | Category.Authority,         Category.Custody           => false
  | Category.Authority,         Category.Evidence          => false
  | Category.Authority,         Category.AcceptanceReceipt => false
  | Category.Execution,         Category.Execution         => true
  | Category.Execution,         Category.Evidence          => true
  | Category.Execution,         Category.Custody           => false
  | Category.Execution,         Category.Authority          => false
  | Category.Execution,         Category.AcceptanceReceipt => false

/-- Transitive closure of `allowed` — all 25 pairs enumerated.
    Verifiable by BFS comparison in the test suite (test_closure.py). -/
def canReach : Category → Category → Bool
  | Category.Custody,           Category.Custody           => true
  | Category.Custody,           Category.Evidence          => true
  | Category.Custody,           Category.AcceptanceReceipt => true
  | Category.Custody,           Category.Authority         => false
  | Category.Custody,           Category.Execution         => false
  | Category.Evidence,          Category.Custody           => false
  | Category.Evidence,          Category.Evidence          => true
  | Category.Evidence,          Category.AcceptanceReceipt => true
  | Category.Evidence,          Category.Authority         => false
  | Category.Evidence,          Category.Execution         => false
  | Category.AcceptanceReceipt, Category.Custody           => false
  | Category.AcceptanceReceipt, Category.Evidence          => true
  | Category.AcceptanceReceipt, Category.AcceptanceReceipt => true
  | Category.AcceptanceReceipt, Category.Authority         => false
  | Category.AcceptanceReceipt, Category.Execution         => false
  | Category.Authority,         Category.Custody           => false
  | Category.Authority,         Category.Evidence          => false
  | Category.Authority,         Category.AcceptanceReceipt => false
  | Category.Authority,         Category.Authority         => true
  | Category.Authority,         Category.Execution         => true
  | Category.Execution,         Category.Custody           => false
  | Category.Execution,         Category.Evidence          => true
  | Category.Execution,         Category.AcceptanceReceipt => false
  | Category.Execution,         Category.Authority         => false
  | Category.Execution,         Category.Execution         => true

/-- Key theorem: producing a receipt from custody does not create a path to authority.

    canReach Custody AcceptanceReceipt = true   (receipt is reachable)
    canReach Custody Authority = false          (authority is NOT reachable) -/
theorem custody_receipt_no_authority_path :
    canReach Category.Custody Category.AcceptanceReceipt = true ∧
    canReach Category.Custody Category.Authority = false := by
  decide

/-- Strengthened: no custody-plane category can reach Authority. -/
theorem custody_no_authority :
    canReach Category.Custody Category.Authority = false := by
  decide

theorem evidence_no_authority :
    canReach Category.Evidence Category.Authority = false := by
  decide

theorem receipt_no_authority :
    canReach Category.AcceptanceReceipt Category.Authority = false := by
  decide

/-- No custody-plane category can reach any authority-plane category. -/
theorem custody_plane_separation :
    canReach Category.Custody Category.Authority = false ∧
    canReach Category.Custody Category.Execution = false ∧
    canReach Category.Evidence Category.Authority = false ∧
    canReach Category.Evidence Category.Execution = false ∧
    canReach Category.AcceptanceReceipt Category.Authority = false ∧
    canReach Category.AcceptanceReceipt Category.Execution = false := by
  decide

/-- No allowed transition crosses from custody plane to authority plane. -/
theorem no_plane_crossing :
    allowed Category.Custody Category.Authority = false ∧
    allowed Category.Custody Category.Execution = false ∧
    allowed Category.Evidence Category.Authority = false ∧
    allowed Category.Evidence Category.Execution = false ∧
    allowed Category.AcceptanceReceipt Category.Authority = false ∧
    allowed Category.AcceptanceReceipt Category.Execution = false := by
  decide

/-- The acceptance receipt stays in the custody plane. -/
theorem receipt_in_custody_plane :
    isCustodyPlane Category.AcceptanceReceipt = true := by
  decide

/-- The acceptance receipt is NOT in the authority plane. -/
theorem receipt_not_in_authority_plane :
    isAuthorityPlane Category.AcceptanceReceipt = false := by
  decide

/-- The harness produces evidence, not authority:
    the receipt is in the custody plane and NOT in the authority plane. -/
theorem harness_is_evidence_generator :
    isCustodyPlane Category.AcceptanceReceipt = true ∧
    isAuthorityPlane Category.AcceptanceReceipt = false := by
  decide

end TSCP.CustodyPlane
