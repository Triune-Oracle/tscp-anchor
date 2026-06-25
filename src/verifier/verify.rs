use crate::verifier::backend::{StubVerifier, Verifier};
use crate::verifier::reason::ReasonCode;
use crate::verifier::types::{ProofEnvelope, VerifyResult};

const SUPPORTED_PROTOCOLS: &[&str] = &["1.0"];
const SUPPORTED_ENGINES: &[&str] = &["0.6.1"];

pub fn verify(envelope: &ProofEnvelope) -> VerifyResult {
    if envelope.protocol.name != "TSCP"
        || !SUPPORTED_PROTOCOLS.contains(&envelope.protocol.version.as_str())
    {
        return VerifyResult::fail(ReasonCode::UnsupportedProtocol);
    }

    if envelope.engine.name != "plonky3"
        || !SUPPORTED_ENGINES.contains(&envelope.engine.version.as_str())
    {
        return VerifyResult::fail(ReasonCode::UnsupportedEngine);
    }

    if !envelope.claim_hash.starts_with("sha256:")
        || envelope.claim_hash.len() != 71
    {
        return VerifyResult::fail(ReasonCode::RejectBinding);
    }

    if envelope.proof_payload.is_empty() {
        return VerifyResult::fail(ReasonCode::RejectSchema);
    }

    let backend = StubVerifier;

    match backend.verify(envelope) {
        Ok(()) => VerifyResult::pass(),
        Err(_) => VerifyResult::fail(ReasonCode::RejectProof),
    }
}
