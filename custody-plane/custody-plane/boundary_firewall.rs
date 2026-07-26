// boundary_firewall.rs
// TSCP Custody Plane — Structural Firewall
//
// Checks FCO (Formal Custody Object) transitions against the transition matrix.
// Validates that no path exists from any custody-plane category to any
// authority-plane category.
//
// Classification: Evidence Generator component (NOT an Authority Generator).
// The firewall reports violations; it does not grant permission.

use std::collections::{HashMap, HashSet};

/// A transition in the custody plane topology.
#[derive(Debug, Clone, PartialEq)]
pub struct Transition {
    pub from: String,
    pub to: String,
    pub allowed: bool,
}

/// The transition matrix — the full topology of allowed/forbidden transitions.
#[derive(Debug, Clone)]
pub struct TransitionMatrix {
    pub categories: HashMap<String, String>, // category → plane ("custody" or "authority")
    pub transitions: Vec<Transition>,
}

/// A Formal Custody Object (FCO).
#[derive(Debug, Clone)]
pub struct FCO {
    pub category: String,
    pub content_hash: String,
    pub parent_hash: Option<String>,
    pub constraints: FCOConstraints,
}

/// Constitutional constraints — all must be false (negative assertions).
#[derive(Debug, Clone, PartialEq)]
pub struct FCOConstraints {
    pub authority_granted: bool,
    pub permission_granted: bool,
    pub jurisdiction_crossed: bool,
}

impl Default for FCOConstraints {
    fn default() -> Self {
        Self {
            authority_granted: false,
            permission_granted: false,
            jurisdiction_crossed: false,
        }
    }
}

/// Firewall check result.
#[derive(Debug, Clone, PartialEq)]
pub struct FirewallResult {
    pub violations: Vec<String>,
    pub clean: bool,
}

/// Reachability check result.
#[derive(Debug, Clone, PartialEq)]
pub struct ReachabilityResult {
    pub authority_reachable: bool,
    pub clean: bool,
    pub path: Option<Vec<String>>,
}

impl TransitionMatrix {
    /// Load from a JSON file (simplified — in production use serde).
    pub fn new() -> Self {
        let mut categories = HashMap::new();
        categories.insert("Custody".to_string(), "custody".to_string());
        categories.insert("Evidence".to_string(), "custody".to_string());
        categories.insert("AcceptanceReceipt".to_string(), "custody".to_string());
        categories.insert("Authority".to_string(), "authority".to_string());
        categories.insert("Execution".to_string(), "authority".to_string());

        let transitions = vec![
            Transition { from: "Custody".into(), to: "Custody".into(), allowed: true },
            Transition { from: "Custody".into(), to: "Evidence".into(), allowed: true },
            Transition { from: "Custody".into(), to: "AcceptanceReceipt".into(), allowed: true },
            Transition { from: "Custody".into(), to: "Authority".into(), allowed: false },
            Transition { from: "Custody".into(), to: "Execution".into(), allowed: false },
            Transition { from: "Evidence".into(), to: "Evidence".into(), allowed: true },
            Transition { from: "Evidence".into(), to: "AcceptanceReceipt".into(), allowed: true },
            Transition { from: "Evidence".into(), to: "Authority".into(), allowed: false },
            Transition { from: "Evidence".into(), to: "Execution".into(), allowed: false },
            Transition { from: "Evidence".into(), to: "Custody".into(), allowed: false },
            Transition { from: "AcceptanceReceipt".into(), to: "AcceptanceReceipt".into(), allowed: true },
            Transition { from: "AcceptanceReceipt".into(), to: "Evidence".into(), allowed: true },
            Transition { from: "AcceptanceReceipt".into(), to: "Custody".into(), allowed: false },
            Transition { from: "AcceptanceReceipt".into(), to: "Authority".into(), allowed: false },
            Transition { from: "AcceptanceReceipt".into(), to: "Execution".into(), allowed: false },
            Transition { from: "Authority".into(), to: "Authority".into(), allowed: true },
            Transition { from: "Authority".into(), to: "Execution".into(), allowed: true },
            Transition { from: "Authority".into(), to: "Custody".into(), allowed: false },
            Transition { from: "Authority".into(), to: "Evidence".into(), allowed: false },
            Transition { from: "Authority".into(), to: "AcceptanceReceipt".into(), allowed: false },
            Transition { from: "Execution".into(), to: "Execution".into(), allowed: true },
            Transition { from: "Execution".into(), to: "Evidence".into(), allowed: true },
            Transition { from: "Execution".into(), to: "Custody".into(), allowed: false },
            Transition { from: "Execution".into(), to: "Authority".into(), allowed: false },
            Transition { from: "Execution".into(), to: "AcceptanceReceipt".into(), allowed: false },
        ];

        Self { categories, transitions }
    }

    /// Check if a specific transition is allowed.
    pub fn check_transition(&self, from: &str, to: &str) -> Option<bool> {
        self.transitions
            .iter()
            .find(|t| t.from == from && t.to == to)
            .map(|t| t.allowed)
    }

    /// Check if a category is unknown (not in the matrix).
    pub fn is_known_category(&self, category: &str) -> bool {
        self.categories.contains_key(category)
    }

    /// Get the plane of a category.
    pub fn plane_of(&self, category: &str) -> Option<&str> {
        self.categories.get(category).map(|s| s.as_str())
    }

    /// Get all categories reachable from `start` via ALLOWED transitions only.
    pub fn reachable_from(&self, start: &str) -> HashSet<String> {
        let mut visited = HashSet::new();
        let mut queue = vec![start.to_string()];

        while let Some(current) = queue.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            for t in &self.transitions {
                if t.from == current && t.allowed && !visited.contains(&t.to) {
                    queue.push(t.to.clone());
                }
            }
        }

        visited
    }

    /// Check if Authority is reachable from `start` via allowed transitions.
    /// Returns the path if reachable, None if not.
    pub fn path_to_authority(&self, start: &str) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        self.dfs_to_authority(start, &mut visited, vec![])
    }

    fn dfs_to_authority(
        &self,
        current: &str,
        visited: &mut HashSet<String>,
        path: Vec<String>,
    ) -> Option<Vec<String>> {
        if visited.contains(current) {
            return None;
        }
        visited.insert(current.to_string());
        let mut path = path;
        path.push(current.to_string());

        if current == "Authority" || current == "Execution" {
            // Only count as authority-reachable if we started from custody plane
            if path.len() > 1 {
                return Some(path);
            }
        }

        for t in &self.transitions {
            if t.from == current && t.allowed {
                if let Some(p) = self.dfs_to_authority(&t.to, visited, path.clone()) {
                    return Some(p);
                }
            }
        }

        None
    }
}

/// The firewall — checks FCO boundary compliance.
pub struct Firewall {
    matrix: TransitionMatrix,
}

impl Firewall {
    pub fn new() -> Self {
        Self {
            matrix: TransitionMatrix::new(),
        }
    }

    /// Check a proposed transition between two FCO categories.
    pub fn check(&self, from_category: &str, to_category: &str) -> FirewallResult {
        let mut violations = Vec::new();

        // Check 1: Both categories must be known
        if !self.matrix.is_known_category(from_category) {
            violations.push(format!("Unknown source category: {}", from_category));
        }
        if !self.matrix.is_known_category(to_category) {
            violations.push(format!("Unknown target category: {}", to_category));
        }
        if !violations.is_empty() {
            return FirewallResult {
                violations,
                clean: false,
            };
        }

        // Check 2: Transition must be in the matrix
        match self.matrix.check_transition(from_category, to_category) {
            None => {
                violations.push(format!(
                    "Transition not in matrix: {} → {}",
                    from_category, to_category
                ));
            }
            Some(false) => {
                violations.push(format!(
                    "Forbidden transition: {} → {}",
                    from_category, to_category
                ));
            }
            Some(true) => {}
        }

        // Check 3: No path from source to authority plane via allowed transitions
        if let Some(path) = self.matrix.path_to_authority(from_category) {
            if from_category != "Authority" && from_category != "Execution" {
                violations.push(format!(
                    "Authority reachable from {}: path = {:?}",
                    from_category, path
                ));
            }
        }

        FirewallResult {
            clean: violations.is_empty(),
            violations,
        }
    }

    /// Check reachability — is authority reachable from a custody-plane category?
    pub fn check_reachability(&self, category: &str) -> ReachabilityResult {
        if let Some(path) = self.matrix.path_to_authority(category) {
            ReachabilityResult {
                authority_reachable: true,
                clean: false,
                path: Some(path),
            }
        } else {
            ReachabilityResult {
                authority_reachable: false,
                clean: true,
                path: None,
            }
        }
    }

    /// Validate FCO constraints — all must be false.
    pub fn validate_constraints(fco: &FCO) -> Vec<String> {
        let mut violations = Vec::new();
        if fco.constraints.authority_granted {
            violations.push("authority_granted is true (must be false)".to_string());
        }
        if fco.constraints.permission_granted {
            violations.push("permission_granted is true (must be false)".to_string());
        }
        if fco.constraints.jurisdiction_crossed {
            violations.push("jurisdiction_crossed is true (must be false)".to_string());
        }
        violations
    }

    /// Get the transition count (for verification).
    pub fn transition_count(&self) -> usize {
        self.matrix.transitions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed_transition() {
        let fw = Firewall::new();
        let result = fw.check("Custody", "AcceptanceReceipt");
        assert!(result.clean, "Custody → AcceptanceReceipt should be allowed");
    }

    #[test]
    fn test_forbidden_transition() {
        let fw = Firewall::new();
        let result = fw.check("Custody", "Authority");
        assert!(!result.clean, "Custody → Authority should be forbidden");
        assert!(
            result.violations.iter().any(|v| v.contains("Forbidden")),
            "Should report forbidden transition"
        );
    }

    #[test]
    fn test_unknown_category() {
        let fw = Firewall::new();
        let result = fw.check("Custody", "Unknown");
        assert!(!result.clean, "Unknown category should be rejected");
    }

    #[test]
    fn test_no_authority_path_from_custody() {
        let fw = Firewall::new();
        let result = fw.check_reachability("Custody");
        assert!(!result.authority_reachable, "Authority must not be reachable from Custody");
        assert!(result.clean);
    }

    #[test]
    fn test_no_authority_path_from_receipt() {
        let fw = Firewall::new();
        let result = fw.check_reachability("AcceptanceReceipt");
        assert!(
            !result.authority_reachable,
            "Authority must not be reachable from AcceptanceReceipt"
        );
        assert!(result.clean);
    }

    #[test]
    fn test_no_authority_path_from_evidence() {
        let fw = Firewall::new();
        let result = fw.check_reachability("Evidence");
        assert!(!result.authority_reachable);
        assert!(result.clean);
    }

    #[test]
    fn test_constraint_violation_detected() {
        let fco = FCO {
            category: "Custody".into(),
            content_hash: "abc".into(),
            parent_hash: None,
            constraints: FCOConstraints {
                authority_granted: true,
                permission_granted: false,
                jurisdiction_crossed: false,
            },
        };
        let violations = Firewall::validate_constraints(&fco);
        assert!(!violations.is_empty(), "Should detect authority_granted=true");
    }

    #[test]
    fn test_constraints_clean() {
        let fco = FCO {
            category: "Custody".into(),
            content_hash: "abc".into(),
            parent_hash: None,
            constraints: FCOConstraints::default(),
        };
        let violations = Firewall::validate_constraints(&fco);
        assert!(violations.is_empty(), "Default constraints should be clean");
    }

    #[test]
    fn test_transition_count() {
        let fw = Firewall::new();
        assert_eq!(fw.transition_count(), 25, "Matrix should have 25 transitions (5×5)");
    }
}
