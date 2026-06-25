use tscp_anchor::prover::prove_receipt;
use tscp_kernel::types::TransitionHash;

fn main() {
    let hash: TransitionHash = [0u8; 32];
    let proof = prove_receipt(&hash);

    println!("{}", hex::encode(proof));
}
