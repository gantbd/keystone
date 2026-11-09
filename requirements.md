# Requirements

- **K1 — Semantic neutrality:** no crate owns or interprets domain-engine behavior.
- **K2 — Focused adoption:** crates are independently usable and do not create a
  mandatory common-runtime dependency.
- **K3 — Explicit bounds:** allocations, parsing work, retained records, counters,
  and diagnostic capture are bounded by caller-selected profiles.
- **K4 — Honest evidence:** refusal, committed, and indeterminate effects remain
  distinct; adapters state their actual mechanism and environment.
- **K5 — Atomic refusal:** mechanism operations validate before publication.
- **K6 — Privacy by construction:** shared diagnostic types redact content and
  secrets unless a domain explicitly opts into disclosure.
- **K7 — Determinism:** semantic helpers read no clock, randomness, filesystem, or
  process state implicitly.
- **K8 — Lawful composition:** algebraic primitives ship exhaustive law tests over
  their finite domains.
- **K9 — Conformance:** adapters are qualified through reusable behavior suites;
  a missing adapter or test environment remains open.

