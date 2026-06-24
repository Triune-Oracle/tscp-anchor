pub mod prover;
use serde::{Deserialize, Serialize};
use tscp_kernel::types::{TransitionReceipt, TransitionHash};
use prover::{prove_receipt, verify_proof};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AnchoredReceipt {
    pub receipt: TransitionReceipt,
    pub receipt_hash: TransitionHash,
    pub stark_proof: Vec<u8>,
}

impl AnchoredReceipt {
    pub fn new(receipt: TransitionReceipt) -> Self {
        let receipt_hash = receipt.hash();
        let stark_proof = prove_receipt(&receipt_hash);
        Self { receipt, receipt_hash, stark_proof }
    }

    pub fn verify(&self) -> bool {
        verify_proof(&self.receipt_hash, &self.stark_proof)
    }
}

pub struct Anchor;

impl Anchor {
    pub fn anchor_receipt(receipt: TransitionReceipt) -> AnchoredReceipt {
        AnchoredReceipt::new(receipt)
    }
}
