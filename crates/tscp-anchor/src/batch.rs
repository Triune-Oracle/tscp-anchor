use crate::AnchoredReceipt;
use sha2::{Sha256, Digest};

pub struct BatchAnchor {
    receipts: Vec<AnchoredReceipt>,
}

impl BatchAnchor {
    pub fn new() -> Self {
        Self { receipts: vec![] }
    }
    
    pub fn add(&mut self, receipt: AnchoredReceipt) {
        self.receipts.push(receipt);
    }
    
    pub fn merkle_root(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        for r in &self.receipts {
            hasher.update(&r.receipt_hash);
        }
        hasher.finalize().into()
    }
    
    pub fn len(&self) -> usize { self.receipts.len() }
}
