# Keystone

Keystone is a Rust workspace of small, reusable mechanisms shared by the bounded
engines in this workspace. It is not an engine and owns no conversation, task,
document, workflow, event, authority, or harness semantics.

The initial crates provide:

- bounded bytes, strings, collections, and checked counters;
- bounded canonical codec building blocks;
- an exhaustive four-value effect-evidence lattice;
- separate checkpoint, append, and immutable-blob storage protocols;
- a truthful volatile checkpoint reference adapter;
- privacy-safe debug wrappers;
- bounded non-authoritative observation;
- explicit lifecycle obligation tracking;
- deterministic clocks, identifiers, and fault plans for tests.

`keystone-storage-directory` is deliberately a scaffold. Filesystem durability is
not inferred from the volatile adapter or protocol tests.

## Boundary

Domain engines own their schemas, commands, refusals, migrations, resource limits,
and semantic store traits. Keystone supplies mechanisms and laws only. Adoption is
per crate and must not force an engine to depend on unrelated mechanisms.

See [requirements](requirements.md), [operating rigor](docs/operating-rigor.md),
[roadmap](docs/roadmap.md), and [traceability](docs/traceability.md).
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

Vellum consumes the narrow `keystone-codec` cursor and byte-field primitives
behind its own versioned wire anti-corruption layer. Vellum owns all schema tags,
bounds, identity and UTF-8 validation, compatibility, and disclosure policy.
The [qualification record](docs/vellum-codec-qualification.md) scopes the golden
and adversarial evidence; it is not a daemon, transport, or general codec-tooling
claim.

## Development

```sh
make check
```

The production workspace is self-contained: a fresh checkout needs no sibling
engine repositories. Cross-project Chronos/Moira qualification is deliberately
standalone and can be run from a checkout containing all three repositories:

```sh
make qualify-chronos-moira
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

