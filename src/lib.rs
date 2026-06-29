//! Builder for explainable evidence chains.
//!
//! An [`EvidenceChain`] is an ordered list of [`EvidenceLink`]s — each link
//! records one verifiable observation, an optional metric, and whether that
//! observation met its threshold. Calling [`EvidenceChain::finalize`] computes
//! an aggregate [`EvidenceStrength`] (pass ratio) across all links.
//!
//! The design is domain-agnostic: the same types work for fraud detection,
//! trading signals, compliance checks, on-chain analysis, or any system where
//! decisions must be auditable and explainable.
//!
//! # Quick start
//!
//! ```
//! use evidence_chain::{EvidenceChain, EvidenceLink, EvidenceCategory};
//!
//! let mut chain = EvidenceChain::new("fraud-detector", "1.0.0");
//!
//! chain.add_link(
//!     EvidenceLink::new(EvidenceCategory::Value, "Transaction above limit", "tx:8f3a")
//!         .with_metric(15_000.0, "USD")
//!         .with_threshold(10_000.0, true),
//! );
//! chain.add_link(
//!     EvidenceLink::new(EvidenceCategory::Behavioral, "New device fingerprint", "device:c91b")
//!         .with_threshold(1.0, true),
//! );
//! chain.add_link(
//!     EvidenceLink::new(EvidenceCategory::Temporal, "Outside normal hours", "2024-03-15T03:42Z")
//!         .with_threshold(1.0, false), // did not meet threshold
//! );
//!
//! chain.finalize();
//!
//! assert_eq!(chain.strength.total_checks, 3);
//! assert_eq!(chain.strength.passed_checks, 2);
//! assert!((chain.strength.ratio - 2.0 / 3.0).abs() < 1e-9);
//! ```
//!
//! # Serialization
//!
//! All types implement [`serde::Serialize`] and [`serde::Deserialize`], so chains
//! can be stored as JSON, embedded in database columns, or sent over the wire.
//!
//! ```
//! use evidence_chain::{EvidenceChain, EvidenceLink, EvidenceCategory};
//!
//! let mut chain = EvidenceChain::new("signal-v1", "2.0.0");
//! chain.add_link(EvidenceLink::new(EvidenceCategory::Structural, "Pattern matched", "ref:001"));
//! chain.finalize();
//!
//! let json = serde_json::to_string(&chain).unwrap();
//! let decoded: EvidenceChain = serde_json::from_str(&json).unwrap();
//! assert_eq!(decoded.heuristic_id, "signal-v1");
//! ```

use serde::{Deserialize, Serialize};

/// Broad category of an evidence observation.
///
/// Use whichever category best describes the nature of the observation.
/// All variants are valid for any domain — the labels are intentionally general.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceCategory {
    /// Shape or composition of the observed entity (counts, ratios, topology).
    Structural,
    /// Time-based observations (age, duration, gaps, timestamps).
    Temporal,
    /// Numerical or monetary observations (amounts, scores, percentages).
    Value,
    /// Pattern or history-based observations (reuse, frequency, sequences).
    Behavioral,
}

/// A single verifiable observation in an [`EvidenceChain`].
///
/// Each link records:
/// - *what* was observed (`observation`)
/// - *where* it can be verified (`reference`)
/// - *how much* was measured (`metric_value` + `metric_unit`)
/// - *whether* it cleared the decision threshold (`threshold`, `threshold_met`)
///
/// Build links with the fluent API:
///
/// ```
/// use evidence_chain::{EvidenceLink, EvidenceCategory};
///
/// let link = EvidenceLink::new(EvidenceCategory::Value, "Score above cutoff", "record:42")
///     .with_metric(0.87, "probability")
///     .with_threshold(0.80, true);
///
/// assert!(link.threshold_met);
/// assert_eq!(link.metric_unit.as_deref(), Some("probability"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceLink {
    /// Broad category of this observation.
    pub category: EvidenceCategory,
    /// Human-readable description of the observed fact.
    pub observation: String,
    /// Verifiable pointer to the source: an ID, hash, URL, timestamp, etc.
    pub reference: String,
    /// Numerical measurement of the observation (optional).
    pub metric_value: Option<f64>,
    /// Unit of `metric_value` (e.g. `"USD"`, `"blocks"`, `"sat/vB"`, `"ratio"`).
    pub metric_unit: Option<String>,
    /// Decision threshold for this criterion (optional).
    pub threshold: Option<f64>,
    /// `true` if `metric_value` satisfies `threshold`, or if no threshold applies.
    pub threshold_met: bool,
}

impl EvidenceLink {
    /// Creates a new link with the minimum required fields.
    ///
    /// `threshold_met` defaults to `true`. Use [`with_threshold`](Self::with_threshold)
    /// to set it explicitly when a numeric threshold applies.
    pub fn new(
        category: EvidenceCategory,
        observation: impl Into<String>,
        reference: impl Into<String>,
    ) -> Self {
        Self {
            category,
            observation: observation.into(),
            reference: reference.into(),
            metric_value: None,
            metric_unit: None,
            threshold: None,
            threshold_met: true,
        }
    }

    /// Attaches a numeric measurement to this link.
    pub fn with_metric(mut self, value: f64, unit: impl Into<String>) -> Self {
        self.metric_value = Some(value);
        self.metric_unit = Some(unit.into());
        self
    }

    /// Records the decision threshold and whether it was met.
    pub fn with_threshold(mut self, threshold: f64, met: bool) -> Self {
        self.threshold = Some(threshold);
        self.threshold_met = met;
        self
    }
}

/// Aggregate pass/fail ratio across all links in an [`EvidenceChain`].
///
/// Computed by [`EvidenceChain::finalize`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceStrength {
    /// Total number of links in the chain.
    pub total_checks: usize,
    /// Number of links where `threshold_met == true`.
    pub passed_checks: usize,
    /// `passed_checks / total_checks`; `0.0` when `total_checks == 0`.
    pub ratio: f64,
}

/// An ordered chain of evidence links for a single decision.
///
/// Build the chain by adding [`EvidenceLink`]s, then call [`finalize`](Self::finalize)
/// to compute the aggregate [`EvidenceStrength`].
///
/// `heuristic_id` and `heuristic_version` identify the rule or model that
/// produced this chain — use any stable identifiers meaningful to your domain.
///
/// # Example
///
/// ```
/// use evidence_chain::{EvidenceChain, EvidenceLink, EvidenceCategory};
///
/// let mut chain = EvidenceChain::new("velocity-check", "1.2.0");
///
/// chain.add_link(
///     EvidenceLink::new(EvidenceCategory::Value, "5 transactions in 10 minutes", "window:abc")
///         .with_metric(5.0, "tx/10min")
///         .with_threshold(3.0, true),
/// );
/// chain.add_link(
///     EvidenceLink::new(EvidenceCategory::Structural, "All transactions to same recipient", "addr:xyz")
///         .with_threshold(1.0, true),
/// );
///
/// chain.finalize();
/// assert_eq!(chain.strength.ratio, 1.0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceChain {
    /// Identifier of the rule or model that produced this chain.
    pub heuristic_id: String,
    /// Version of the rule or model.
    pub heuristic_version: String,
    /// Ordered list of evidence observations.
    pub links: Vec<EvidenceLink>,
    /// Aggregate strength — populated by [`finalize`](Self::finalize).
    pub strength: EvidenceStrength,
}

impl EvidenceChain {
    /// Creates an empty chain for the given rule identifier and version.
    pub fn new(heuristic_id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            heuristic_id: heuristic_id.into(),
            heuristic_version: version.into(),
            links: Vec::new(),
            strength: EvidenceStrength {
                total_checks: 0,
                passed_checks: 0,
                ratio: 0.0,
            },
        }
    }

    /// Appends an evidence link to the chain.
    pub fn add_link(&mut self, link: EvidenceLink) {
        self.links.push(link);
    }

    /// Recomputes [`EvidenceStrength`] from the current links.
    ///
    /// Idempotent — safe to call multiple times. Call after all links have
    /// been added to get an accurate `strength`.
    pub fn finalize(&mut self) {
        let total = self.links.len();
        let passed = self.links.iter().filter(|l| l.threshold_met).count();
        self.strength = EvidenceStrength {
            total_checks: total,
            passed_checks: passed,
            ratio: if total == 0 {
                0.0
            } else {
                passed as f64 / total as f64
            },
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn link(met: bool) -> EvidenceLink {
        EvidenceLink::new(EvidenceCategory::Structural, "observation", "ref:1")
            .with_metric(15.0, "units")
            .with_threshold(10.0, met)
    }

    #[test]
    fn empty_chain_has_zero_strength() {
        let mut chain = EvidenceChain::new("rule-001", "1.0.0");
        chain.finalize();
        assert_eq!(chain.strength.total_checks, 0);
        assert_eq!(chain.strength.passed_checks, 0);
        assert_eq!(chain.strength.ratio, 0.0);
    }

    #[test]
    fn all_pass_gives_ratio_one() {
        let mut chain = EvidenceChain::new("rule-001", "1.0.0");
        chain.add_link(link(true));
        chain.add_link(link(true));
        chain.add_link(link(true));
        chain.finalize();
        assert_eq!(chain.strength.total_checks, 3);
        assert_eq!(chain.strength.passed_checks, 3);
        assert_eq!(chain.strength.ratio, 1.0);
    }

    #[test]
    fn partial_pass_gives_correct_ratio() {
        let mut chain = EvidenceChain::new("rule-001", "1.0.0");
        chain.add_link(link(true));
        chain.add_link(link(true));
        chain.add_link(link(false));
        chain.add_link(link(false));
        chain.finalize();
        assert_eq!(chain.strength.total_checks, 4);
        assert_eq!(chain.strength.passed_checks, 2);
        assert!((chain.strength.ratio - 0.5).abs() < 1e-9);
    }

    #[test]
    fn add_link_increments_count() {
        let mut chain = EvidenceChain::new("rule-001", "1.0.0");
        assert_eq!(chain.links.len(), 0);
        chain.add_link(link(true));
        assert_eq!(chain.links.len(), 1);
        chain.add_link(link(false));
        assert_eq!(chain.links.len(), 2);
    }

    #[test]
    fn finalize_is_idempotent() {
        let mut chain = EvidenceChain::new("rule-001", "1.0.0");
        chain.add_link(link(true));
        chain.add_link(link(false));
        chain.finalize();
        let ratio = chain.strength.ratio;
        chain.finalize();
        assert!((chain.strength.ratio - ratio).abs() < 1e-9);
    }

    #[test]
    fn evidence_link_builder_fields() {
        let link = EvidenceLink::new(EvidenceCategory::Temporal, "Age exceeded limit", "record:42")
            .with_metric(60_000.0, "blocks")
            .with_threshold(157_680.0, true);
        assert_eq!(link.observation, "Age exceeded limit");
        assert_eq!(link.metric_value, Some(60_000.0));
        assert_eq!(link.metric_unit.as_deref(), Some("blocks"));
        assert_eq!(link.threshold, Some(157_680.0));
        assert!(link.threshold_met);
    }

    #[test]
    fn evidence_link_default_threshold_met_is_true() {
        let link = EvidenceLink::new(EvidenceCategory::Behavioral, "Pattern observed", "ref:x");
        assert!(link.threshold_met);
        assert!(link.threshold.is_none());
        assert!(link.metric_value.is_none());
    }

    #[test]
    fn evidence_link_serialization_roundtrip() {
        let link = EvidenceLink::new(EvidenceCategory::Value, "Score above cutoff", "record:99")
            .with_metric(0.91, "probability")
            .with_threshold(0.80, true);
        let json = serde_json::to_string(&link).unwrap();
        let decoded: EvidenceLink = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.observation, "Score above cutoff");
        assert_eq!(decoded.metric_value, Some(0.91));
        assert!(decoded.threshold_met);
    }

    #[test]
    fn evidence_chain_serialization_roundtrip() {
        let mut chain = EvidenceChain::new("velocity-check", "1.0.0");
        chain.add_link(link(true));
        chain.add_link(link(false));
        chain.finalize();
        let json = serde_json::to_string(&chain).unwrap();
        let decoded: EvidenceChain = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.heuristic_id, "velocity-check");
        assert_eq!(decoded.strength.total_checks, 2);
        assert!((decoded.strength.ratio - 0.5).abs() < 1e-9);
    }

    #[test]
    fn evidence_category_serializes_to_snake_case() {
        let cases = [
            (EvidenceCategory::Structural, "structural"),
            (EvidenceCategory::Temporal, "temporal"),
            (EvidenceCategory::Value, "value"),
            (EvidenceCategory::Behavioral, "behavioral"),
        ];
        for (variant, expected) in &cases {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(json, format!("\"{}\"", expected));
            let decoded: EvidenceCategory = serde_json::from_str(&json).unwrap();
            assert_eq!(&decoded, variant);
        }
    }

    #[test]
    fn chain_with_no_failed_links_has_full_strength() {
        let mut chain = EvidenceChain::new("compliance-check", "3.1.0");
        for _ in 0..10 {
            chain.add_link(link(true));
        }
        chain.finalize();
        assert_eq!(chain.strength.ratio, 1.0);
        assert_eq!(chain.strength.passed_checks, 10);
    }
}
