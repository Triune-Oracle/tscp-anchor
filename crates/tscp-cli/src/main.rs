use tscp_kernel::{event::EventEnvelope, replay::ReplayEngine, types::GENESIS_STATE};
use tscp_anchor::Anchor;

fn main() {
    println!("TSCP CLI v0.1 - Kernel→Anchor pipe");

    let mut engine = ReplayEngine::new(1);
    let mut parent = GENESIS_STATE;

    for i in 1..=3 {
        let event = EventEnvelope {
            event_id: [i; 16],
            parent_state_hash: parent,
            payload_hash: [i+10; 32],
            logical_time: i as u64,
        };

        let receipt = engine.apply(&event).unwrap();
        parent = receipt.child_state_hash; // chain it!

        let anchored = Anchor::anchor_receipt(receipt.clone());

        println!("\nEvent {}:", i);
        println!(" receipt.hash: {:x?}", &anchored.receipt_hash[..8]);
        println!(" proof: {:x?}", &anchored.stark_proof[..8]);
        println!(" verify: {}", anchored.verify());
    }
}
