# Keystone production roadmap

Status: active extraction and consumer qualification  
Scope: small, independently adoptable Rust mechanisms shared by sovereign engines

Keystone is not a common runtime, service framework, domain model or repository
for code that merely looks reusable. It owns bounded mechanisms and executable
laws below engine authority. A consumer keeps its schemas, commands, outcomes,
refusals, identities, migrations, retry and recovery policy.

## Admission gate

A production mechanism enters Keystone only when all of the following hold:

1. At least two real consumers demonstrate the same law and failure meaning.
2. Its contract can be stated without task, document, room, grant, workflow,
   participant or agent vocabulary.
3. Bounds, allocation behavior and uncertainty are explicit at the public API.
4. It reads no ambient clock, randomness, filesystem or process state unless it
   is a narrowly named adapter whose physical profile is part of the contract.
5. It does not acquire authorization, retry, routing, reconciliation, recovery
   or lifecycle policy from its consumers.
6. It ships with law, boundary and adversarial tests plus consumer migration
   evidence. Similar names or implementations are not qualification.
7. Adoption remains per crate and does not pull unrelated Keystone facilities
   into a consumer.

If the second consumer is missing or the failure meanings differ, keep the code
with its current owner and record the candidate here instead of extracting it.

## Current status

Workspace membership does not imply qualification. `qualification-status.json`
is the boundary-checked profile classification: qualified profiles have matching
multi-consumer evidence, adopted profiles have narrow real-consumer evidence,
and reference profiles remain comparison or test material. Every claim names its
profile and evidence; there is deliberately no crate-wide maturity label.

### Qualified profiles

- Four-value effect evidence, qualified against Chronos and consumed by Moira.
- Bounded request/reply framing with pre-dispatch versus outcome-unknown
  evidence, consumed by Aegis, Hermes and Moira.
- Identity-guarded private Unix listener publication and cleanup, consumed by
  Aegis, Hermes and Moira under a trusted-parent profile.
- Volatile and private-directory checkpoint profiles with distinct commit and
  load evidence. The local Unix profile is qualified through Hermes and Aegis.
- Dev-only Cargo license-declaration baseline verifier, consumed by Aegis,
  Hermes and Moira while each repository owns its accepted policy and evidence.

### Narrowly adopted profiles

- Canonical bounded codec cursor and byte fields behind consumer-owned schemas.
- Bounded non-authoritative FIFO capture and selected observation arithmetic.
- Exact-input replay relation, adopted narrowly by Vellum; Agora retains its
  different allocation, persistence and live-limit contract.

### Reference/incubating surfaces

- General bounds, lifecycle, redaction, memory storage and test-kit crates.
- Append and immutable-blob contracts, available only through
  `keystone-storage`'s `incubating-append-blob` feature. They have no production
  qualification claim and remain candidates until two consumers agree on
  ordering, conflict, durability and recovery evidence.

## Wave 1 — close current extractions

### Durable publication gate

Extract the mechanical law shared by Aegis and Hermes:

1. Prepare and encode a complete candidate before storage is offered.
2. Offer it against the exact last-confirmed revision.
3. Accept only an exact durable receipt for the proposed revision.
4. Publish candidate state and outcome only after that receipt.
5. Fence the live instance after any store error, refusal, indeterminate result,
   weak receipt or mismatched receipt.
6. Treat revision exhaustion as a pre-offer refusal, not storage uncertainty.

Keystone owns the confirmed-revision/fence mechanism and its laws. Aegis and
Hermes retain candidate construction, checkpoint bytes, recovery, domain errors,
observations and decisions about which operations require publication.

Status: complete for the declared serialized checkpoint-owner profile.
`keystone-storage::CommitGate` is adopted by the Aegis and Hermes durable hosts;
`make qualify-aegis-hermes` runs both consumers' independent fault matrices, and
the qualification record states the shared law and explicit nonclaims.

### Observation arithmetic

Migrate matching sticky counters and duration totals in Aegis and Hermes to
`keystone-observe`. Domain signal structures and operation labels remain local.
Before each migration, verify the value-after-overflow law: some existing code
clamps to `Duration::MAX`, while other code preserves the previous exact value.
Do not hide that difference behind one type.

Status: matching arithmetic adopted. Aegis uses Keystone counters and duration
totals. Hermes uses Keystone counters but deliberately retains its local
last-exact-duration behavior; the divergent overflow scenario is executable and
is not represented as a shared Keystone law.

### Replay adoption

Status: held with Agora. Its V1, V2, search, export and retirement registries
share replay/conflict/capacity outcomes with `keystone-replay`, but not one
mechanism contract. Agora allocates incrementally, V2 persists canonical ordered
receipts and uses a policy-selected live ceiling, and retirement installs durable
non-reexecution tombstones. `keystone-replay` preallocates one fixed volatile
vector. Preserve these differences; do not migrate merely to share the outcome
names.

### Codec assurance kit

Status: reference implementation. Feature-gated dev-only helpers cover golden
round trips, every-truncation refusal, selected trailing bytes, bounded
deterministic single-bit mutation and accepted-is-canonical checks. Helpers
invoke consumer-supplied codecs and assertions; they do not define a universal
schema, wire envelope, error vocabulary or encoder object model. Keep the
profile reference-only until real consumers adopt the exact test contract.

## Wave 2 — narrowly shared local mechanisms

Compare Chronos, Vellum, Moira and Keystone after their active refactors, then
extract only independently useful pieces:

- private directory and file validation with no-follow opens;
- owner, mode, link-count and device/inode identity checks;
- lifetime advisory locks and replacement detection;
- guarded Unix socket publication and identity-checked cleanup;
- bounded framed I/O with declared deadlines;
- phase-aware client evidence distinguishing `not_sent` from
  `outcome_unknown`.

Status: the common server-side length-prefixed frame loop and its dispatch-phase
evidence are extracted and qualified through Aegis, Hermes and Moira. Socket
publication and identity-guarded cleanup are separately extracted for the same
consumers. Parent-directory trust, deadlines, authentication, client connection
phases, stale-socket recovery and retry policy remain consumer-owned; neither
crate claims them.

Client phase evidence was compared and is deliberately held with its owners.
Aegis classifies concrete Unix connect, ready-frame, write and reply phases;
Agora couples uncertainty to an exact pending request and replay namespace; and
Chronos treats return from its transport-port invocation as the uncertainty
boundary. Hermes and Moira do not provide a second matching client contract.
The shared words `not_sent` and `outcome_unknown` are therefore not sufficient
to create a Keystone client or generic uncertainty crate.

Do not create a `keystone-local-runtime`. Checkpoint retention, socket routing,
wire schemas, daemon lifecycle and retry policy stay with their components.

## Wave 3 — evidence-driven candidates

- Append and immutable-blob reference adapters, only when two consumers agree
  on ordering, conflict, durability and recovery evidence.
- Lifecycle-ledger qualification through a second real daemon or controlled
  runtime whose obligation laws match Vellum's.
- Bounded state-exploration helpers for finite deterministic transition systems.
- Small protocol-envelope value primitives only after peer DTO comparison shows
  exact matching bounds and meanings. No universal command bus.

## Hold with current owners

The following are deliberately not Keystone work without a second matching
contract:

- Vellum atomic user-file publication and save authority.
- Moira SQLite identity, transaction and recovery behavior.
- Agora's checksum-protected directory record as a replacement for Keystone's
  distinct checksum-free checkpoint profile.
- Chronos checkpoint retention, migration, time and recovery attribution.
- Bounded process execution, filesystem watching and debounce.
- Release manifests, artifact activation and rollback policy.
- Checksums or cryptographic integrity as a generic facility.
- Capability meaning, authentication, authorization or service discovery.
- Unix peer-credential retrieval was compared after Aegis, Hermes and Moira
  adopted separate local adapters. Keep it with those owners: Aegis owns fresh
  session/admin mapping, Hermes maps a volatile subject, and Moira maps only
  ordinary read/mutate facts while delegating operator authority separately.
  The security claim is inseparable from each deployment's identity policy. Do
  not create a Keystone authentication or common-identity crate.
- Any engine's domain schemas, identities, commands, outcomes or refusals.

## Required evidence for each wave

- Focused unit and exhaustive finite-domain laws where possible.
- Allocation, boundary and refusal-identity scenarios.
- Fault injection at every publication boundary.
- Content-safe `Debug` and observation checks.
- STRIDE/LINDDUN review for new state or physical adapters.
- Green Keystone gates and green gates in every migrated consumer.
- Updated traceability with exact claims and explicit nonclaims.

