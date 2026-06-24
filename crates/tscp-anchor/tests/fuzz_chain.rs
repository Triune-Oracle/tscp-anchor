#[test]
fn test_1000_events() {
use tscp_anchor::Anchor;
use tscp_kernel::{event::EventEnvelope, replay::ReplayEngine, types::GENESIS_STATE};
let mut engine = ReplayEngine::new(1); let mut parent = GENESIS_STATE;
for i in 0..1000 { let e = EventEnvelope { event_id: [(i%256) as u8; 16], parent_state_hash: parent, payload_hash: [((i+1)%256) as u8; 32], logical_time: i, }; let receipt = engine.apply(&e).unwrap(); parent = receipt.child_state_hash; let anchored = Anchor::anchor_receipt(receipt.clone()); assert!(anchored.verify()); if i % 100 == 0 { println!("{} events chained, still hot!", i); } } println!("1000 events — Japan money!"); }
