use tscp_anchor::Anchor;
use tscp_kernel::{event::EventEnvelope, replay::ReplayEngine, types::GENESIS_STATE};

#[test]
fn test_kernel_to_anchor() {
    let e1 = EventEnvelope {
        event_id: [1;16],
        parent_state_hash: GENESIS_STATE,
        payload_hash: [2;32],
        logical_time: 1
    };
    let mut eng = ReplayEngine::new(1);
    let receipt = eng.apply(&e1).unwrap().clone();

    let anchored = Anchor::anchor_receipt(receipt.clone());
    assert_eq!(anchored.receipt_hash, receipt.hash());
    assert!(anchored.verify(), "STARK proof should verify");
    println!("✓ Receipt anchored with proof: {:x?}", &anchored.stark_proof[..8]);
}
