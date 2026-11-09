# Roadmap

## Phase 0 — reference primitives

- Establish bounds, effect laws, redaction, bounded recorder, lifecycle ledger,
  deterministic testkit, codec cursor, and volatile checkpoint behavior.
- Keep directory storage explicitly unavailable.

## Phase 1 — consumer qualification

- Compare primitives against Chronos without changing Chronos behavior.
- Select one new project as a second consumer for each candidate crate.
- Add consumer-side compatibility scenarios before extracting production code.

## Phase 2 — durable mechanisms

- Close the directory checkpoint contract and fault matrix.
- Add subprocess crash tests and declared host/filesystem evidence.
- Consider append and blob reference adapters only when Hermes or artifact work
  supplies concrete requirements.

## Phase 3 — tooling

- Add codec golden-fixture tooling, conformance macros, parser robustness helpers,
  and reusable bounded-state exploration.
- Keep CI and supply-chain templates separate from runtime dependencies.

