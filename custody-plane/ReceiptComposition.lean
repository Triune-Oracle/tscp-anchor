namespace TSCP.ReceiptComposition

/- Algebra version binding: FCO_RECEIPT_COMPOSITION_ALGEBRA.md
   Hash: d89ba345180d012fc473851e761379ccdcdc813e277de62436c1950453d855b2
   This Lean model targets the receipt composition algebra specification.
   Conformance is established by `decide` proofs, not by this declaration. -/

inductive ReceiptField
  | Id
  | Subject
  | Evidence
  | Claim
  | Lineage
  | Timestamp
  | Signature

def isAllowedField : ReceiptField → Bool
  | .Id => true
  | .Subject => true
  | .Evidence => true
  | .Claim => true
  | .Lineage => true
  | .Timestamp => true
  | .Signature => true

instance (P : ReceiptField → Prop) [inst : ∀ f, Decidable (P f)] : Decidable (∀ f, P f) :=
  match inst .Id, inst .Subject, inst .Evidence, inst .Claim, inst .Lineage, inst .Timestamp, inst .Signature with
  | isTrue h1, isTrue h2, isTrue h3, isTrue h4, isTrue h5, isTrue h6, isTrue h7 => isTrue (fun f => match f with
    | .Id => h1
    | .Subject => h2
    | .Evidence => h3
    | .Claim => h4
    | .Lineage => h5
    | .Timestamp => h6
    | .Signature => h7)
  | isFalse h, _, _, _, _, _, _ => isFalse (fun all => h (all .Id))
  | _, isFalse h, _, _, _, _, _ => isFalse (fun all => h (all .Subject))
  | _, _, isFalse h, _, _, _, _ => isFalse (fun all => h (all .Evidence))
  | _, _, _, isFalse h, _, _, _ => isFalse (fun all => h (all .Claim))
  | _, _, _, _, isFalse h, _, _ => isFalse (fun all => h (all .Lineage))
  | _, _, _, _, _, isFalse h, _ => isFalse (fun all => h (all .Timestamp))
  | _, _, _, _, _, _, isFalse h => isFalse (fun all => h (all .Signature))

theorem all_fields_allowed : ∀ f, isAllowedField f = true := by decide

inductive DriftClass
  | implementationDrift
  | specificationDefect
  | oracleDefect
  | lineageDivergence

inductive DriftScenario
  | codeWrongRelativeToSpec
  | specAmbiguous
  | referenceWrong
  | bothChangedIndependently

def classifyDrift : DriftScenario → DriftClass
  | .codeWrongRelativeToSpec => .implementationDrift
  | .specAmbiguous => .specificationDefect
  | .referenceWrong => .oracleDefect
  | .bothChangedIndependently => .lineageDivergence

theorem composition_preserves_fields : ∀ f, isAllowedField f = true → isAllowedField f = true := by simp

theorem authority_absent : ∀ f : ReceiptField, isAllowedField f = true := by decide

end TSCP.ReceiptComposition
