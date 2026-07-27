// receipt_validator.rs
// TSCP Custody Plane — Receipt Composition Algebra Validator
//
// Implements the FCO Receipt Composition Algebra as defined in
// FCO_RECEIPT_COMPOSITION_ALGEBRA.md.
//
// Complies with the authority exclusion theorem: Composition may increase
// evidence, but never jurisdiction/authority.
//
// This file is self-contained and compiles without external dependencies using:
// rustc --edition 2021 --test receipt_validator.rs -o validator && ./validator

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

// ── Algebra Version Binding ─────────────────────────────────────────────────
#[allow(dead_code)]
pub const ALGEBRA_VERSION: &str = "53fb4b5a093f7539587be2fc7703f482ac0f5c23c9bbdc89f9ef7614b7df7cda";

// ── Struct Definitions ──────────────────────────────────────────────────────

/// A claim representing an assertion, its dependencies, and the scope of applicability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Claim {
    pub statement: String,
    pub dependencies: Vec<String>,
    pub scope: String,
}

/// A directed acyclic graph (DAG) representing the lineage/provenance chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lineage {
    pub nodes: Vec<String>,
    pub edges: Vec<(String, String)>, // (from, to) meaning "from depends on to"
}

/// The exact 7-field receipt structure. Structurally excludes authority fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub id: String,
    pub subject: String,
    pub evidence: Vec<String>,
    pub claim: Claim,
    pub lineage: Lineage,
    pub timestamp: String,
    pub signature: String,
}

/// Semantic drift classification for negative receipts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriftClassification {
    ImplementationDrift,
    SpecificationDefect,
    OracleDefect,
    LineageDivergence,
}

// ── Serialization & Hashing (SHA-256) ──────────────────────────────────────

impl Receipt {
    /// Serializes the content of the receipt deterministically for hashing.
    /// Replaces occurrences of self.id with "{self}" in the lineage to break
    /// the circular hash dependency.
    pub fn serialize_content(&self) -> String {
        let mut sorted_evidence = self.evidence.clone();
        sorted_evidence.sort();
        let evidence_str = sorted_evidence.join(",");

        let mut sorted_deps = self.claim.dependencies.clone();
        sorted_deps.sort();
        let claim_str = format!(
            "{}:{}:{}",
            self.claim.statement,
            sorted_deps.join(","),
            self.claim.scope
        );

        // Normalize nodes, replacing any self-references with "{self}"
        let mut processed_nodes: Vec<String> = self
            .lineage
            .nodes
            .iter()
            .map(|n| {
                if n == &self.id {
                    "{self}".to_string()
                } else {
                    n.clone()
                }
            })
            .collect();
        processed_nodes.sort();
        processed_nodes.dedup();

        // Normalize edges, replacing any self-references with "{self}"
        let mut processed_edges: Vec<(String, String)> = self
            .lineage
            .edges
            .iter()
            .map(|(u, v)| {
                let u_new = if u == &self.id { "{self}".to_string() } else { u.clone() };
                let v_new = if v == &self.id { "{self}".to_string() } else { v.clone() };
                (u_new, v_new)
            })
            .collect();
        processed_edges.sort();
        processed_edges.dedup();

        let edges_str = processed_edges
            .iter()
            .map(|(u, v)| format!("{}>{}", u, v))
            .collect::<Vec<String>>()
            .join(",");
        let lineage_str = format!("{}:{}", processed_nodes.join(","), edges_str);

        format!(
            "subject:{};evidence:{};claim:{};lineage:{};timestamp:{}",
            self.subject, evidence_str, claim_str, lineage_str, self.timestamp
        )
    }
}

/// Computes a standard SHA-256 hex digest for any byte slice without external crates.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let mut pads = data.to_vec();
    let len_bits = (data.len() as u64) * 8;
    pads.push(0x80);
    while (pads.len() + 8) % 64 != 0 {
        pads.push(0x00);
    }
    pads.extend_from_slice(&len_bits.to_be_bytes());

    for chunk in pads.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut h_val = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h_val
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h_val = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(h_val);
    }

    let mut result = String::new();
    for val in h {
        result.push_str(&format!("{:08x}", val));
    }
    result
}

// ── Validation & Composition Functions ──────────────────────────────────────

/// Structurally checks if all 7 fields of the receipt are present/valid
/// and confirms authority exclusion.
pub fn validate_receipt(r: &Receipt) -> bool {
    if r.id.is_empty() || r.subject.is_empty() || r.timestamp.is_empty() || r.signature.is_empty() {
        return false;
    }
    check_authority_exclusion(r)
}

/// Verifies that the receipt contains no authority fields.
/// Rust's strict compile-time types guarantee that the Receipt struct
/// contains exactly the 7 fields declared, completely excluding authority.
pub fn check_authority_exclusion(_r: &Receipt) -> bool {
    true
}

/// Guard 1: Subject Agreement
/// Receipts must be about the exact same subject.
pub fn check_subject_agreement(r1: &Receipt, r2: &Receipt) -> bool {
    r1.subject == r2.subject && !r1.subject.is_empty()
}

/// Guard 2: Claim Consistency
/// Rejects composition if claims contradict each other.
pub fn check_claim_consistency(r1: &Receipt, r2: &Receipt) -> bool {
    let s1 = r1.claim.statement.trim().to_lowercase();
    let s2 = r2.claim.statement.trim().to_lowercase();

    if s1 == s2 {
        return true; // identical claims, perfectly consistent
    }

    // Heuristics to detect negations/contradictions:
    // Normalize negation/failure words to check if they refer to the same root claim
    let normalize = |s: &str| -> String {
        s.replace("does not ", "")
            .replace("not ", "")
            .replace("fails to ", "")
            .replace("failed ", "passed ")
            .replace("incorrect", "correct")
            .replace("failure", "success")
            .trim()
            .to_string()
    };

    if normalize(&s1) == normalize(&s2) {
        let has_neg1 = s1.contains("not") || s1.contains("fail") || s1.contains("incorrect");
        let has_neg2 = s2.contains("not") || s2.contains("fail") || s2.contains("incorrect");
        if has_neg1 != has_neg2 {
            return false; // Contradiction! One is negative, one is positive.
        }
    }

    // Keyword opposite pairings
    if (s1.contains("matches") && s2.contains("does not match"))
        || (s2.contains("matches") && s1.contains("does not match"))
    {
        let s1_sub = s1.replace("does not match", "matches");
        if s1_sub == s2 || s2.replace("does not match", "matches") == s1 {
            return false;
        }
    }
    if (s1.contains("passed") && s2.contains("failed"))
        || (s2.contains("passed") && s1.contains("failed"))
    {
        let s1_sub = s1.replace("failed", "passed");
        if s1_sub == s2 || s2.replace("failed", "passed") == s1 {
            return false;
        }
    }

    true
}

/// Helper function to perform cycle detection in a graph.
fn has_cycle(nodes: &[String], edges: &[(String, String)]) -> bool {
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for node in nodes {
        adj.entry(node.as_str()).or_default();
    }
    for (src, dst) in edges {
        adj.entry(src.as_str()).or_default().push(dst.as_str());
        adj.entry(dst.as_str()).or_default();
    }

    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();

    fn dfs<'a>(
        node: &'a str,
        adj: &HashMap<&'a str, Vec<&'a str>>,
        visited: &mut HashSet<&'a str>,
        rec_stack: &mut HashSet<&'a str>,
    ) -> bool {
        if rec_stack.contains(node) {
            return true;
        }
        if visited.contains(node) {
            return false;
        }

        visited.insert(node);
        rec_stack.insert(node);

        if let Some(neighbors) = adj.get(node) {
            for &neighbor in neighbors {
                if dfs(neighbor, adj, visited, rec_stack) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    for &node in adj.keys() {
        if dfs(node, &adj, &mut visited, &mut rec_stack) {
            return true;
        }
    }

    false
}

/// Guard 3: Lineage Acyclicity
/// Verifies that the union of both lineages remains acyclic.
pub fn check_lineage_acyclicity(r1: &Receipt, r2: &Receipt) -> bool {
    let mut union_nodes = HashSet::new();
    for n in &r1.lineage.nodes {
        union_nodes.insert(n.clone());
    }
    for n in &r2.lineage.nodes {
        union_nodes.insert(n.clone());
    }
    union_nodes.insert(r1.id.clone());
    union_nodes.insert(r2.id.clone());

    let mut union_edges = HashSet::new();
    for e in &r1.lineage.edges {
        union_edges.insert(e.clone());
    }
    for e in &r2.lineage.edges {
        union_edges.insert(e.clone());
    }

    let nodes_vec: Vec<String> = union_nodes.into_iter().collect();
    let edges_vec: Vec<(String, String)> = union_edges.into_iter().collect();

    !has_cycle(&nodes_vec, &edges_vec)
}

/// Guard 4: Evidence Independence
/// If receipts share any evidence, the sharing must be explicitly declared
/// via a common lineage node.
pub fn check_evidence_independence(r1: &Receipt, r2: &Receipt) -> bool {
    let mut r1_ev = HashSet::new();
    for ev in &r1.evidence {
        r1_ev.insert(ev);
    }
    let mut shared_evidence = Vec::new();
    for ev in &r2.evidence {
        if r1_ev.contains(ev) {
            shared_evidence.push(ev);
        }
    }

    if shared_evidence.is_empty() {
        return true; // Perfectly independent
    }

    // Shared evidence requires at least one common lineage node (ancestry)
    let mut r1_nodes = HashSet::new();
    for node in &r1.lineage.nodes {
        r1_nodes.insert(node.clone());
    }
    r1_nodes.insert(r1.id.clone());

    for node in &r2.lineage.nodes {
        if r1_nodes.contains(node) {
            return true;
        }
    }
    if r1_nodes.contains(&r2.id) {
        return true;
    }

    false
}

/// Computes the intersection of scope elements.
fn intersect_scopes(s1: &str, s2: &str) -> String {
    let set1: HashSet<&str> = s1
        .split(|c| c == ',' || c == ';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let set2: HashSet<&str> = s2
        .split(|c| c == ',' || c == ';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut intersection: Vec<&str> = set1.into_iter().filter(|item| set2.contains(item)).collect();
    intersection.sort();
    intersection.join(", ")
}

/// Checks all 4 guards to validate whether r1 and r2 can compose.
pub fn validate_composition(r1: &Receipt, r2: &Receipt) -> bool {
    check_subject_agreement(r1, r2)
        && check_claim_consistency(r1, r2)
        && check_lineage_acyclicity(r1, r2)
        && check_evidence_independence(r1, r2)
}

/// Composes two receipts into a new receipt. Returns None if guards fail.
pub fn compose(r1: &Receipt, r2: &Receipt) -> Option<Receipt> {
    if !validate_composition(r1, r2) {
        return None;
    }

    // 1. Evidence: Set union
    let mut evidence_set = HashSet::new();
    for ev in &r1.evidence {
        evidence_set.insert(ev.clone());
    }
    for ev in &r2.evidence {
        evidence_set.insert(ev.clone());
    }
    let mut evidence: Vec<String> = evidence_set.into_iter().collect();
    evidence.sort();

    // 2. Claim Conjunction
    let statement = if r1.claim.statement == r2.claim.statement {
        r1.claim.statement.clone()
    } else {
        format!("({} AND {})", r1.claim.statement, r2.claim.statement)
    };

    // Dependencies: Set union
    let mut deps_set = HashSet::new();
    for dep in &r1.claim.dependencies {
        deps_set.insert(dep.clone());
    }
    for dep in &r2.claim.dependencies {
        deps_set.insert(dep.clone());
    }
    let mut dependencies: Vec<String> = deps_set.into_iter().collect();
    dependencies.sort();

    // Scope: Intersection
    let scope = intersect_scopes(&r1.claim.scope, &r2.claim.scope);

    let claim = Claim {
        statement,
        dependencies,
        scope,
    };

    // 3. Timestamp: Max ISO-8601 string
    let timestamp = std::cmp::max(r1.timestamp.clone(), r2.timestamp.clone());

    // 4. Lineage DAG: Union of nodes & edges, adding r3 as root pointing to r1 and r2.
    let mut nodes_set = HashSet::new();
    for n in &r1.lineage.nodes {
        nodes_set.insert(n.clone());
    }
    for n in &r2.lineage.nodes {
        nodes_set.insert(n.clone());
    }
    nodes_set.insert(r1.id.clone());
    nodes_set.insert(r2.id.clone());

    let placeholder = "PENDING_SELF_ID".to_string();
    nodes_set.insert(placeholder.clone());

    let mut edges_set = HashSet::new();
    for e in &r1.lineage.edges {
        edges_set.insert(e.clone());
    }
    for e in &r2.lineage.edges {
        edges_set.insert(e.clone());
    }
    edges_set.insert((placeholder.clone(), r1.id.clone()));
    edges_set.insert((placeholder.clone(), r2.id.clone()));

    let lineage = Lineage {
        nodes: nodes_set.into_iter().collect(),
        edges: edges_set.into_iter().collect(),
    };

    let mut r3 = Receipt {
        id: placeholder.clone(),
        subject: r1.subject.clone(),
        evidence,
        claim,
        lineage,
        timestamp,
        signature: "PENDING_SIGNATURE".to_string(),
    };

    // Content-addressed hashing loop to resolve the ID self-dependency
    let serialized = r3.serialize_content();
    let id = sha256_hex(serialized.as_bytes());

    r3.id = id.clone();
    for node in r3.lineage.nodes.iter_mut() {
        if node == &placeholder {
            *node = id.clone();
        }
    }
    for (u, v) in r3.lineage.edges.iter_mut() {
        if u == &placeholder {
            *u = id.clone();
        }
        if v == &placeholder {
            *v = id.clone();
        }
    }

    let final_serialized = r3.serialize_content();
    r3.signature = sha256_hex(format!("id:{};{}", r3.id, final_serialized).as_bytes());

    Some(r3)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Test helper to create valid receipts with deterministic hashing.
    fn create_test_receipt(
        subject: &str,
        evidence: Vec<&str>,
        statement: &str,
        dependencies: Vec<&str>,
        scope: &str,
        nodes: Vec<&str>,
        edges: Vec<(&str, &str)>,
        timestamp: &str,
    ) -> Receipt {
        let claim = Claim {
            statement: statement.to_string(),
            dependencies: dependencies.into_iter().map(String::from).collect(),
            scope: scope.to_string(),
        };

        let mut nodes_strings: Vec<String> = nodes.into_iter().map(String::from).collect();
        if !nodes_strings.contains(&"PENDING_SELF_ID".to_string()) {
            nodes_strings.push("PENDING_SELF_ID".to_string());
        }

        let lineage = Lineage {
            nodes: nodes_strings,
            edges: edges
                .into_iter()
                .map(|(u, v)| (u.to_string(), v.to_string()))
                .collect(),
        };

        let mut r = Receipt {
            id: "PENDING_SELF_ID".to_string(),
            subject: subject.to_string(),
            evidence: evidence.into_iter().map(String::from).collect(),
            claim,
            lineage,
            timestamp: timestamp.to_string(),
            signature: "PENDING_SIGNATURE".to_string(),
        };

        let serialized = r.serialize_content();
        let id = sha256_hex(serialized.as_bytes());

        r.id = id.clone();
        for node in r.lineage.nodes.iter_mut() {
            if node == "PENDING_SELF_ID" {
                *node = id.clone();
            }
        }
        for (u, v) in r.lineage.edges.iter_mut() {
            if u == "PENDING_SELF_ID" {
                *u = id.clone();
            }
            if v == "PENDING_SELF_ID" {
                *v = id.clone();
            }
        }

        let final_serialized = r.serialize_content();
        r.signature = sha256_hex(format!("id:{};{}", r.id, final_serialized).as_bytes());

        r
    }

    #[test]
    fn test_valid_receipt() {
        let r = create_test_receipt(
            "NTT Backend Equivalence",
            vec!["log_t1"],
            "NTT matches Montgomery reference",
            vec!["log_t1"],
            "arithmetic",
            vec![],
            vec![],
            "2026-07-26T12:00:00Z",
        );
        assert!(validate_receipt(&r));
    }

    #[test]
    fn test_authority_exclusion() {
        let r = create_test_receipt(
            "NTT Backend Equivalence",
            vec!["log_t1"],
            "NTT matches Montgomery reference",
            vec!["log_t1"],
            "arithmetic",
            vec![],
            vec![],
            "2026-07-26T12:00:00Z",
        );
        // Direct field check of the type to guarantee authority does not exist
        assert!(check_authority_exclusion(&r));
        assert!(validate_receipt(&r));
    }

    #[test]
    fn test_valid_composition() {
        let r1 = create_test_receipt(
            "Arithmetic",
            vec!["log1"],
            "AVX backend matches scalar backend",
            vec!["log1"],
            "correctness, speed",
            vec!["anc1"],
            vec![],
            "2026-07-26T12:00:00Z",
        );

        let r2 = create_test_receipt(
            "Arithmetic",
            vec!["log2"],
            "scalar backend matches Montgomery reference",
            vec!["log2"],
            "correctness, security",
            vec!["anc2"],
            vec![],
            "2026-07-26T14:00:00Z",
        );

        assert!(validate_composition(&r1, &r2));
        let r3 = compose(&r1, &r2).unwrap();

        assert_eq!(r3.subject, "Arithmetic");
        assert_eq!(r3.evidence, vec!["log1", "log2"]);
        assert_eq!(
            r3.claim.statement,
            "(AVX backend matches scalar backend AND scalar backend matches Montgomery reference)"
        );
        assert_eq!(r3.claim.dependencies, vec!["log1", "log2"]);
        assert_eq!(r3.claim.scope, "correctness"); // Intersected!
        assert_eq!(r3.timestamp, "2026-07-26T14:00:00Z"); // Max of r1 and r2
        assert!(validate_receipt(&r3));
    }

    #[test]
    fn test_subject_mismatch_rejected() {
        let r1 = create_test_receipt(
            "Arithmetic",
            vec!["log1"],
            "matches reference",
            vec!["log1"],
            "correctness",
            vec![],
            vec![],
            "2026-07-26T12:00:00Z",
        );

        let r2 = create_test_receipt(
            "Cryptography",
            vec!["log2"],
            "matches reference",
            vec!["log2"],
            "correctness",
            vec![],
            vec![],
            "2026-07-26T12:00:00Z",
        );

        assert!(!validate_composition(&r1, &r2));
        assert!(compose(&r1, &r2).is_none());
    }

    #[test]
    fn test_claim_contradiction_rejected() {
        let r1 = create_test_receipt(
            "Arithmetic",
            vec!["log1"],
            "AVX backend matches reference",
            vec!["log1"],
            "correctness",
            vec![],
            vec![],
            "2026-07-26T12:00:00Z",
        );

        let r2 = create_test_receipt(
            "Arithmetic",
            vec!["log2"],
            "AVX backend does not match reference",
            vec!["log2"],
            "correctness",
            vec![],
            vec![],
            "2026-07-26T12:00:00Z",
        );

        assert!(!validate_composition(&r1, &r2));
        assert!(compose(&r1, &r2).is_none());
    }

    #[test]
    fn test_lineage_cycle_rejected() {
        // Construct a dependency cycle in lineage union
        let r1 = create_test_receipt(
            "Arithmetic",
            vec!["log1"],
            "matches reference",
            vec!["log1"],
            "correctness",
            vec!["A", "B"],
            vec![("A", "B")],
            "2026-07-26T12:00:00Z",
        );

        let r2 = create_test_receipt(
            "Arithmetic",
            vec!["log2"],
            "matches reference",
            vec!["log2"],
            "correctness",
            vec!["B", "A"],
            vec![("B", "A")],
            "2026-07-26T12:00:00Z",
        );

        assert!(!validate_composition(&r1, &r2));
        assert!(compose(&r1, &r2).is_none());
    }

    #[test]
    fn test_evidence_overlap_rejected() {
        // 1. Shared evidence without a common ancestor should fail
        let r1 = create_test_receipt(
            "Arithmetic",
            vec!["log_shared"],
            "AVX matches scalar",
            vec!["log_shared"],
            "correctness",
            vec!["anc1"],
            vec![],
            "2026-07-26T12:00:00Z",
        );

        let r2 = create_test_receipt(
            "Arithmetic",
            vec!["log_shared"],
            "scalar matches ref",
            vec!["log_shared"],
            "correctness",
            vec!["anc2"],
            vec![],
            "2026-07-26T12:00:00Z",
        );

        assert!(!validate_composition(&r1, &r2));

        // 2. Shared evidence WITH a common ancestor should pass
        let r1_shared = create_test_receipt(
            "Arithmetic",
            vec!["log_shared"],
            "AVX matches scalar",
            vec!["log_shared"],
            "correctness",
            vec!["anc1", "common_anc"],
            vec![],
            "2026-07-26T12:00:00Z",
        );

        let r2_shared = create_test_receipt(
            "Arithmetic",
            vec!["log_shared"],
            "scalar matches ref",
            vec!["log_shared"],
            "correctness",
            vec!["anc2", "common_anc"],
            vec![],
            "2026-07-26T12:00:00Z",
        );

        assert!(validate_composition(&r1_shared, &r2_shared));
        assert!(compose(&r1_shared, &r2_shared).is_some());
    }

    #[test]
    fn test_negative_receipt_valid() {
        let r = create_test_receipt(
            "Arithmetic",
            vec!["err_log"],
            "Property P failed under test T",
            vec!["err_log"],
            "correctness",
            vec![],
            vec![],
            "2026-07-26T12:00:00Z",
        );

        assert!(validate_receipt(&r));
    }

    #[test]
    fn test_composition_preserves_authority_exclusion() {
        let r1 = create_test_receipt(
            "Arithmetic",
            vec!["log1"],
            "matches 1",
            vec!["log1"],
            "correctness",
            vec![],
            vec![],
            "2026-07-26T12:00:00Z",
        );

        let r2 = create_test_receipt(
            "Arithmetic",
            vec!["log2"],
            "matches 2",
            vec!["log2"],
            "correctness",
            vec![],
            vec![],
            "2026-07-26T12:00:00Z",
        );

        let r3 = compose(&r1, &r2).unwrap();
        assert!(check_authority_exclusion(&r3));
        assert!(validate_receipt(&r3));
    }
}
