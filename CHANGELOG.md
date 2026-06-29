# Changelog

## [0.1.0] - 2026-06-29

### Added
- `EvidenceChain`: ordered chain of evidence links with aggregate strength computation
- `EvidenceLink`: single verifiable observation with optional metric and threshold
- `EvidenceCategory`: four broad categories (Structural, Temporal, Value, Behavioral)
- `EvidenceStrength`: pass ratio computed by `EvidenceChain::finalize`
- Full serde support (serialize/deserialize to JSON and any serde-compatible format)
- Examples: `fraud_detection`, `trading_signal`
