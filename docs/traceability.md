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
| K9 conformance | volatile and local-directory checkpoint adapters | separate strength, CAS/fault/subprocess and domain restart tests | Qualified only for declared local Unix profile |

## Consumer qualification

| Mechanism | Reference | Second consumer | Evidence | Status |
| --- | --- | --- | --- | --- |
| Effect knowledge | Chronos local lattice | Moira direct dependency/re-export | exhaustive cross-project mapping and operation comparison | Qualified |
| Bounded request/reply framing | Aegis local daemon | Hermes and Moira local daemons | shared mechanism laws plus all three consumer connection suites | Qualified for four-byte big-endian length-prefixed serial request/reply serving |
| Private Unix listener ownership | Aegis local daemon | Hermes and Moira local daemons | mechanism identity/mode/replacement laws plus all three consumer Unix listener suites | Qualified for trusted-parent mode-0600 publication and device/inode-guarded cleanup |
| FIFO diagnostic capture | Vellum recorder storage | Vellum direct dependency; Keystone append-only compatibility facade | shared queue boundary/clone/FIFO/loss tests; Vellum instrumentation, measurement and timing scenarios | Narrow Vellum adoption; no general cross-engine qualification |
| Bounded codec cursor and byte fields | Keystone reference tests | Vellum V1 wire codec | golden octets, all truncations, exact limits, invalid versions/tags/IDs/UTF-8, closed variant round trips | Narrow Vellum qualification |
| Exact-input retained replay | Keystone reference tests | Vellum service | four closed decisions, existing-key-before-capacity, fixed preallocation, explicit owner cleanup and content-safe debug | Narrow Vellum adoption; Agora comparison rejected because allocation, persistence and live-limit laws differ |
| Revisioned local checkpoint | Hermes native durable host | Aegis native durable host | independent domain checkpoint/recovery tests, matching strong receipt/load and uncertainty contract | Qualified for private exclusive local Unix profile |
| Durable publication gate | Aegis durable host | Hermes durable host | `make qualify-aegis-hermes`: exact durable receipt, weak/wrong receipt, refusal/error, before/after uncertainty, pre-offer refusal and recovery scenarios | Qualified for one serialized checkpoint owner; recovery remains domain-owned |
| Sticky observation arithmetic | Aegis durable-host counters and duration totals | Hermes durable-host counters | Keystone accumulator laws plus both host overflow/observation-neutrality suites; Hermes non-clamping duration counterexample | Counter law qualified; clamping duration law adopted by Aegis only |
| Bounds, general observation, clock, lifecycle, redaction | Chronos candidates | none selected | explicit mismatch analysis | Not generally qualified |

The local profile includes process-exit and injected publication-boundary evidence,
not physical power-loss behavior, cryptographic integrity, production performance
or downstream compatibility
beyond the specifically qualified/adopted mechanisms above.

