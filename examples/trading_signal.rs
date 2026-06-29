//! Example: building an evidence chain for a trading signal.
//!
//! cargo run --example trading_signal

use evidence_chain::{EvidenceCategory, EvidenceChain, EvidenceLink};

fn main() {
    let mut chain = EvidenceChain::new("breakout-signal-v2", "2.1.0");

    chain.add_link(
        EvidenceLink::new(
            EvidenceCategory::Value,
            "Price closed above 20-day resistance",
            "BTCUSDT:2024-03-15",
        )
        .with_metric(68_450.0, "USD")
        .with_threshold(67_800.0, true),
    );

    chain.add_link(
        EvidenceLink::new(
            EvidenceCategory::Structural,
            "Volume 2.3x above 20-day average",
            "volume:2024-03-15",
        )
        .with_metric(2.3, "ratio")
        .with_threshold(1.5, true),
    );

    chain.add_link(
        EvidenceLink::new(
            EvidenceCategory::Temporal,
            "Breakout occurred in first 2 hours of NY session",
            "session:NY-open",
        )
        .with_threshold(1.0, true),
    );

    chain.add_link(
        EvidenceLink::new(
            EvidenceCategory::Behavioral,
            "RSI not overbought at breakout",
            "rsi:63.2",
        )
        .with_metric(63.2, "RSI")
        .with_threshold(70.0, true), // met: RSI is below 70
    );

    chain.add_link(
        EvidenceLink::new(
            EvidenceCategory::Value,
            "Funding rate negative — longs not overcrowded",
            "funding:2024-03-15",
        )
        .with_metric(-0.01, "%")
        .with_threshold(0.0, false), // not met: positive funding preferred
    );

    chain.finalize();

    let confidence = match chain.strength.ratio {
        r if r >= 0.8 => "HIGH",
        r if r >= 0.6 => "MEDIUM",
        _ => "LOW",
    };

    println!("Signal:     {} v{}", chain.rule_id, chain.rule_version);
    println!(
        "Confidence: {} ({}/{} checks, {:.0}%)",
        confidence,
        chain.strength.passed_checks,
        chain.strength.total_checks,
        chain.strength.ratio * 100.0
    );
    println!();

    for link in &chain.links {
        let status = if link.threshold_met { "✓" } else { "✗" };
        print!("  [{status}] {}", link.observation);
        if let (Some(val), Some(unit)) = (link.metric_value, &link.metric_unit) {
            print!(" → {val} {unit}");
        }
        println!();
    }
}
