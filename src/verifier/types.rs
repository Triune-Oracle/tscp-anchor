use serde::{Deserialize, Serialize};

use crate::verifier::reason::ReasonCode;

#[derive(Debug, Deserialize)]
pub struct Protocol {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct Engine {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct ProofEnvelope {
    pub protocol: Protocol,
    pub engine: Engine,
    pub claim_hash: String,
    pub proof_payload: String,
    pub metadata_hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VerifyResult {
    pub status: String,
    pub reasoncode: String,
}

impl VerifyResult {
    pub fn fail(reason: ReasonCode) -> Self {
        Self {
            status: "FAIL".to_string(),
            reasoncode: reason.wire().to_string(),
        }
    }

    pub fn pass() -> Self {
        Self {
            status: "PASS".to_string(),
            reasoncode: ReasonCode::VersionAccepted.wire().to_string(),
        }
    }
}
