use crate::verifier::types::ProofEnvelope;

pub trait Verifier {
    fn verify(&self, envelope: &ProofEnvelope) -> Result<(), VerifyError>;
}

#[derive(Debug)]
pub struct VerifyError;

pub struct StubVerifier;

impl Verifier for StubVerifier {
    fn verify(&self, _envelope: &ProofEnvelope) -> Result<(), VerifyError> {
        Err(VerifyError)
    }
}
