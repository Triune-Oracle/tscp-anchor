#[derive(Debug, Clone)]
pub enum ReasonCode {
    VersionAccepted,
    RejectSchema,
    UnsupportedProtocol,
    UnsupportedEngine,
    RejectBinding,
    RejectProof,
}

impl ReasonCode {
    pub fn wire(&self) -> &'static str {
        match self {
            ReasonCode::VersionAccepted => "VERSIONACCEPTED",
            ReasonCode::RejectSchema => "REJECTSCHEMA",
            ReasonCode::UnsupportedProtocol => "UNSUPPORTEDPROTOCOL",
            ReasonCode::UnsupportedEngine => "UNSUPPORTEDENGINE",
            ReasonCode::RejectBinding => "REJECTBINDING",
            ReasonCode::RejectProof => "REJECTPROOF",
        }
    }
}
