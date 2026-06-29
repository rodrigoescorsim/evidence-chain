//! Example: building an evidence chain for a fraud detection rule.
//!
//! cargo run --example fraud_detection

use evidence_chain::{EvidenceCategory, EvidenceChain, EvidenceLink};

fn main() {
    let mut chain = EvidenceChain::new("fraud-velocity-v1", "1.0.0");

    chain.add_link(
        EvidenceLink::new(
            EvidenceCategory::Value,
            "Transaction amount above daily limit",
            "tx:8f3a2c",
        )
        .with_metric(15_000.0, "USD")
        .with_threshold(10_000.0, true),
    );

    chain.add_link(
        EvidenceLink::new(
            EvidenceCategory::Behavioral,
            "New device fingerprint — first transaction from this device",
            "device:c91b44",
        )
        .with_threshold(1.0, true),
    );

    chain.add_link(
        EvidenceLink::new(
            EvidenceCategory::Temporal,
            "Transaction initiated outside normal operating hours (03:42 UTC)",
            "2024-03-15T03:42:00Z",
        )
        .with_threshold(1.0, false), // not met — within acceptable window
    );

    chain.add_link(
        EvidenceLink::new(
            EvidenceCategory::Structural,
            "Recipient is a first-time payee",
            "payee:d7f01a",
        )
        .with_threshold(1.0, true),
    );

    chain.finalize();

    println!("Rule:    {} v{}", chain.heuristic_id, chain.heuristic_version);
    println!(
        "Result:  {}/{} checks passed ({:.0}%)",
        chain.strength.passed_checks,
        chain.strength.total_checks,
        chain.strength.ratio * 100.0
    );
    println!();

    for (i, link) in chain.links.iter().enumerate() {
        let status = if link.threshold_met { "✓" } else { "✗" };
        print!("  [{status}] [{:?}] {}", link.category, link.observation);
        if let (Some(val), Some(unit)) = (link.metric_value, &link.metric_unit) {
            print!(" ({val} {unit})");
        }
        if let Some(threshold) = link.threshold {
            print!(" [threshold: {threshold}]");
        }
        println!(" — {}", link.reference);
        let _ = i;
    }
}
