use crate::verifier::types::ProofEnvelope;

use tscp_anchor::prover::verify_proof;
use tscp_kernel::types::TransitionHash;

pub trait Verifier {
    fn verify(&self, envelope: &ProofEnvelope) -> Result<(), VerifyError>;
}

#[derive(Debug)]
pub struct VerifyError;

pub struct AnchorVerifier;

impl Verifier for AnchorVerifier {
    fn verify(&self, envelope: &ProofEnvelope) -> Result<(), VerifyError> {
        let hex = envelope
            .claim_hash
            .strip_prefix("sha256:")
            .ok_or(VerifyError)?;

        if hex.len() != 64 {
            return Err(VerifyError);
        }

        let mut hash = [0u8; 32];

        for i in 0..32 {
            hash[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).map_err(|_| VerifyError)?;
        }

        let proof = hex::decode(&envelope.proof_payload).map_err(|_| VerifyError)?;

        if verify_proof(&TransitionHash::from(hash), &proof) {
            Ok(())
        } else {
            Err(VerifyError)
        }
    }
}
