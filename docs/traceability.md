# Traceability ledger

| Requirement | Implementation | Evidence | Status |
| --- | --- | --- | --- |
| K1 neutrality | focused generic crates | dependency graph and APIs | Initial |
| K2 adoption | independent workspace packages | per-crate manifests | Initial |
| K3 bounds | bounds, codec, recorder, stores | unit boundary scenarios | Initial |
| K4 evidence | effect lattice and storage outcomes | exhaustive laws and protocol tests | Initial volatile scope |
| K5 refusal | validate-before-mutate helpers | atomicity scenarios | Initial |
| K6 privacy | redacted/content wrappers | debug disclosure tests | Initial |
| K7 determinism | explicit clock/ID/fault inputs | source boundary and tests | Initial |
| K8 laws | effect lattice | exhaustive finite-domain and Chronos correspondence tests | Satisfied for current lattice |
| K9 conformance | storage behavior helpers | volatile adapter only | Open for durable adapters |

## Consumer qualification

| Mechanism | Reference | Second consumer | Evidence | Status |
| --- | --- | --- | --- | --- |
| Effect knowledge | Chronos local lattice | Moira direct dependency/re-export | exhaustive cross-project mapping and operation comparison | Qualified |
| FIFO diagnostic capture | Vellum recorder storage | Vellum direct dependency; Keystone append-only compatibility facade | shared queue boundary/clone/FIFO/loss tests; Vellum instrumentation, measurement and timing scenarios | Narrow Vellum adoption; no general cross-engine qualification |
| Bounds, codec, storage, observation, clock, lifecycle, redaction | Chronos candidates | none selected | explicit mismatch analysis | Not qualified |

No current evidence establishes filesystem crash durability, power-loss behavior,
cryptographic integrity, production performance, or downstream compatibility
beyond the specifically qualified/adopted mechanisms above.

