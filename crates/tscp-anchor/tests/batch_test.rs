use tscp_anchor::{Anchor, BatchAnchor};
use tscp_kernel::{event::EventEnvelope, replay::ReplayEngine, types::GENESIS_STATE};

#[test]
fn test_batch_10() {
    let mut engine = ReplayEngine::new(1);
    let mut batch = BatchAnchor::new();
    let mut parent = GENESIS_STATE;

    for i in 0..10 {
        let e = EventEnvelope {
            event_id: [i;16],
            parent_state_hash: parent,
            payload_hash: [i+1;32],
            logical_time: i as u64,
        };
        let receipt = engine.apply(&e).unwrap().clone();
        parent = receipt.child_state_hash;
        batch.add(Anchor::anchor_receipt(receipt));
    }

    let root = batch.merkle_root();
    println!("Batch of {} anchored", batch.len());
    println!("Merkle root: {:x?}", &root[..8]);
    assert_eq!(batch.len(), 10);
}
