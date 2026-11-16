# Keystone

Keystone is a Rust workspace of small, reusable mechanisms shared by the bounded
engines in this workspace. It is not an engine and owns no conversation, task,
document, workflow, event, authority, or harness semantics.

The workspace contains qualified, narrowly adopted, and reference mechanisms;
membership alone is not a production qualification claim. The machine-checked,
profile-level classification and its evidence links are in
[`qualification-status.json`](qualification-status.json). A crate containing one
qualified profile does not confer that status on its other APIs.
The crates provide:

- bounded bytes, strings, collections, and checked counters;
- bounded canonical codec building blocks;
- bounded request/reply framing with explicit dispatch-phase evidence;
- identity-guarded private Unix listener publication and cleanup;
- an exhaustive four-value effect-evidence lattice;
- qualified checkpoint protocols plus explicitly feature-gated, incubating
  append and immutable-blob candidate contracts;
- a truthful volatile checkpoint reference adapter;
- privacy-safe debug wrappers;
- bounded non-authoritative observation;
- bounded exact-input replay retention without eviction or retry policy;
- explicit lifecycle obligation tracking;
- deterministic clocks, identifiers, and fault plans for tests.

`keystone-storage-directory` now implements one bounded checkpoint register per
private Unix directory, with lifetime OS locking, revision CAS, staged replacement
and explicit file/directory barriers. Its durability profile and nonclaims are
documented in [local checkpoint storage](docs/local-checkpoint.md). Volatile
receipts remain explicitly weaker than durable commit **and load** evidence.

`keystone-storage::CommitGate` extracts the exact durable-receipt publication
law shared by Aegis and Hermes. It advances only for the proposed durable
revision and otherwise enters an absorbing fence after the store offer. Candidate
state, checkpoint encoding, recovery and domain outcomes remain consumer-owned;
the [qualification record](docs/aegis-hermes-publication-gate.md) states the
evidence and nonclaims.

## Boundary

Domain engines own their schemas, commands, refusals, migrations, resource limits,
and semantic store traits. Keystone supplies mechanisms and laws only. Adoption is
per crate and must not force an engine to depend on unrelated mechanisms.
`qualified` means the declared profile has matching multi-consumer evidence;
`adopted` means a narrow real-consumer profile exists but the general admission
gate is not yet closed; `reference` means comparison/test material only.

See [requirements](requirements.md), [operating rigor](docs/operating-rigor.md),
the [production roadmap](ROADMAP.production.md), the focused
[implementation roadmap](docs/roadmap.md), and [traceability](docs/traceability.md).
The first real-consumer decision is recorded in the
[Chronos and Moira qualification](docs/chronos-qualification.md).

`keystone-observe::CaptureBuffer` extracts Vellum's preallocated FIFO diagnostic
storage: fallible construction, drop-newest overload, explicit disabled capture,
owned draining and saturating loss counts. Vellum consumes this mechanism while
retaining its own event vocabulary, timing and measurement counters. Keystone's
existing append-only `Recorder` also delegates to it and retains its API, including
zero-capacity overload counting. Vellum instead maps zero capacity to disabled
capture. Generic event payload memory/destructors remain the consumer's concern;
buffer `Debug` does not expose payloads. This adoption does not qualify Chronos's
different observation contract or any unrelated Keystone mechanism.

Moira also consumes `CaptureBuffer` for optional command observations, retaining
its own exhaustive command/refusal vocabulary, counters and workflow gauges.
Its enabled zero-capacity recorder counts overload; explicit disabled capture does
not. Neither adoption introduces a clock, callback or domain semantics here.

Aegis, Hermes and Moira consume `keystone-framing` for their matching bounded
four-byte length-prefixed request/reply loop. Each retains its authentication,
session, codec, command, deadline, daemon lifecycle and retry policy. The shared
mechanism distinguishes failures before dispatch from reply loss after a domain
outcome exists; it does not decide whether or how an unknown outcome is retried.

Their private local daemons also consume `keystone-unix-listener` for the exact
shared socket-path ownership law: refuse an existing leaf, publish mode `0600`,
and remove only the device/inode identity created by this owner. Directory trust,
peer authentication, deadlines, stale-socket recovery and daemon lifecycle stay
with each application.

Vellum consumes the narrow `keystone-codec` cursor and byte-field primitives
behind its own versioned wire anti-corruption layer. Vellum owns all schema tags,
bounds, identity and UTF-8 validation, compatibility, and disclosure policy.
The [qualification record](docs/vellum-codec-qualification.md) scopes the golden
and adversarial evidence; it is not a daemon, transport, or general codec-tooling
claim.

Vellum consumes the four-outcome `keystone-replay` relation: new, matching
replay, changed-input conflict, and absent-key full. Existing keys are resolved
before capacity, storage is fallibly preallocated, and overload never evicts
retained history. Agora has the same decision vocabulary but deliberately keeps
its incrementally allocated, canonically persisted and dynamically bounded
registries; matching outcomes alone do not establish a shared mechanism.

## Development

```sh
make check
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for scope and validation expectations.
Security-sensitive findings should follow [SECURITY.md](SECURITY.md) rather
than being opened publicly.

The production workspace is self-contained: a fresh checkout needs no sibling
engine repositories. Cross-project qualification is deliberately standalone
and can be run from a checkout containing the relevant sibling repositories:

```sh
make qualify-aegis-hermes
make qualify-chronos-moira
make qualify-framing
make qualify-unix-listener
```

## Distribution

Keystone is source-distributed from GitHub and is not published to crates.io.
Consumers select only the mechanisms they use and pin one reviewed commit, for
example:

```toml
[dependencies]
keystone-codec = { git = "https://github.com/gantbd/keystone", rev = "<full-commit-sha>" }
```

Tags may identify reviewed repository states, but consumers retain the immutable
commit pin in their lockfile. Keystone has no installation binary and grants no
domain authority merely by being linked.

Changes are recorded in [CHANGELOG.md](CHANGELOG.md). Keystone is available
under either the Apache License 2.0 or the MIT License, at your option.

