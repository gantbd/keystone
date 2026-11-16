# Roadmap

## Phase 0 — reference primitives

- Establish bounds, effect laws, redaction, bounded recorder, lifecycle ledger,
  deterministic testkit, codec cursor, and volatile checkpoint behavior.
- [Complete] Keep volatile and durable acknowledgment strength distinct.

## Phase 1 — consumer qualification

- Compare primitives against Chronos without changing Chronos behavior.
- Select one new project as a second consumer for each candidate crate.
- Add consumer-side compatibility scenarios before extracting production code.
- [Complete] Record maturity per named profile with checked evidence links;
  never qualify a mixed-status crate as a whole.

## Phase 2 — durable mechanisms

- [Complete for local Unix profile] Directory checkpoint contract, fault matrix,
  subprocess exit/ownership tests and declared filesystem evidence.
- [Complete for the declared serialized-owner profile] Extract and
  cross-qualify the Aegis/Hermes exact durable receipt gate and absorbing
  post-offer uncertainty fence.
- Consider append and blob reference adapters only when Hermes or artifact work
  supplies concrete requirements.

## Phase 3 — tooling

- [Reference implementation] Add feature-gated golden round-trip, complete
  truncation, selected trailing-byte, bounded single-bit mutation and
  accepted-is-canonical helpers. Keep them reference-only until consumers adopt
  the exact test contract; conformance macros and parser policy remain open.
- [Complete] Add a consumer-owned Cargo license-declaration baseline verifier;
  keep allowlists, legal review and advisory policy with each consumer.
- Keep CI and supply-chain templates separate from runtime dependencies.

## Phase 4 — local composition mechanisms

- [Complete for the declared request/reply profile] Extract bounded four-byte
  length-prefixed framing and pre-dispatch versus outcome-unknown evidence from
  Aegis, Hermes and Moira without absorbing their codecs, services or daemons.
- [Complete for the trusted-parent local Unix profile] Extract listener
  publication, mode and identity-guarded cleanup without absorbing peer
  authentication, socket recovery, deadlines or daemon lifecycle.

