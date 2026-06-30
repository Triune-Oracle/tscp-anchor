use super::EdiaSnapshot;

#[derive(Clone, Debug)]
pub struct AdmissionDecision {
    pub admit: bool,
    pub reason: &'static str,
}

pub struct AdmissionController {
    pub max_inflight: u64,
    pub max_avg_proof_ms: u64,
}

impl AdmissionController {
    pub fn new(max_inflight: u64, max_avg_proof_ms: u64) -> Self {
        Self {
            max_inflight,
            max_avg_proof_ms,
        }
    }

    pub fn decide(&self, snapshot: &EdiaSnapshot) -> AdmissionDecision {
        if !snapshot.invariant_holds {
            return AdmissionDecision {
                admit: false,
                reason: "invariant_failure",
            };
        }

        if snapshot.pending_requests >= self.max_inflight {
            return AdmissionDecision {
                admit: false,
                reason: "backpressure",
            };
        }

        if snapshot.avg_proof_ms > self.max_avg_proof_ms {
            return AdmissionDecision {
                admit: false,
                reason: "latency_budget",
            };
        }

        AdmissionDecision {
            admit: true,
            reason: "healthy",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_backpressure() {
        let c = AdmissionController::new(4, 1000);

        let s = EdiaSnapshot {
            pending_requests: 4,
            completed_proofs: 0,
            avg_proof_ms: 0,
            last_proof_ms: 0,
            invariant_holds: true,
        };

        assert!(!c.decide(&s).admit);
    }

    #[test]
    fn rejects_invariant_break() {
        let c = AdmissionController::new(4, 1000);

        let s = EdiaSnapshot {
            pending_requests: 0,
            completed_proofs: 0,
            avg_proof_ms: 0,
            last_proof_ms: 0,
            invariant_holds: false,
        };

        assert!(!c.decide(&s).admit);
    }

    #[test]
    fn admits_healthy_state() {
        let c = AdmissionController::new(4, 1000);

        let s = EdiaSnapshot {
            pending_requests: 0,
            completed_proofs: 0,
            avg_proof_ms: 10,
            last_proof_ms: 10,
            invariant_holds: true,
        };

        assert!(c.decide(&s).admit);
    }
}
