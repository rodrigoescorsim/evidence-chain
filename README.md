# evidence-chain

[![Crates.io](https://img.shields.io/crates/v/evidence-chain)](https://crates.io/crates/evidence-chain)
[![Docs.rs](https://docs.rs/evidence-chain/badge.svg)](https://docs.rs/evidence-chain)
[![CI](https://github.com/rodrigoescorsim/evidence-chain/actions/workflows/ci.yml/badge.svg)](https://github.com/rodrigoescorsim/evidence-chain/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Builder for explainable evidence chains in Rust.

An `EvidenceChain` is an ordered list of verifiable observations (`EvidenceLink`s), each with an optional numeric metric and a pass/fail threshold. Calling `finalize()` computes an aggregate pass ratio across all links.

**Domain-agnostic** — the same types work for fraud detection, trading signals, compliance checks, on-chain analysis, or any system where decisions must be auditable and explainable.

## Installation

```toml
[dependencies]
evidence-chain = "0.1"
```

## Usage

```rust
use evidence_chain::{EvidenceChain, EvidenceLink, EvidenceCategory};

let mut chain = EvidenceChain::new("fraud-detector", "1.0.0");

chain.add_link(
    EvidenceLink::new(EvidenceCategory::Value, "Amount above daily limit", "tx:8f3a")
        .with_metric(15_000.0, "USD")
        .with_threshold(10_000.0, true),
);
chain.add_link(
    EvidenceLink::new(EvidenceCategory::Behavioral, "New device fingerprint", "device:c91b")
        .with_threshold(1.0, true),
);
chain.add_link(
    EvidenceLink::new(EvidenceCategory::Temporal, "Outside normal hours", "2024-03-15T03:42Z")
        .with_threshold(1.0, false),
);

chain.finalize();

println!("{}/{} checks passed", chain.strength.passed_checks, chain.strength.total_checks);
// → 2/3 checks passed
```

## Core types

| Type | Description |
|------|-------------|
| `EvidenceChain` | Ordered list of links + aggregate strength |
| `EvidenceLink` | Single observation with metric and threshold |
| `EvidenceCategory` | Structural / Temporal / Value / Behavioral |
| `EvidenceStrength` | `total_checks`, `passed_checks`, `ratio` |

## Serialization

All types implement `serde::Serialize` and `serde::Deserialize`:

```rust
let json = serde_json::to_string(&chain)?;
let decoded: EvidenceChain = serde_json::from_str(&json)?;
```

## Examples

```sh
cargo run --example fraud_detection
cargo run --example trading_signal
```

## License

MIT — see [LICENSE](LICENSE).

## Author

[Rodrigo Escorsim](https://github.com/rodrigoescorsim) · [cachesnap.com](https://cachesnap.com)
