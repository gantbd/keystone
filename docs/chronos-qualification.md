# Chronos and Moira qualification

Date: 2026-11-08. This record compares current source contracts; it is not a broad
claim that Keystone can replace Chronos internals.

## Selected migration: effect knowledge

Chronos and Keystone both define the same four-element diamond:

```text
        Conflict
       /        \
   Failed     Completed
       \        /
        Unknown
```

The qualification crate exhaustively checks the four-value bijection, bottom, top,
all 16 join results, all 16 meet results, and all 16 induced-order results. Moira
now consumes and re-exports Keystone's type directly as `EffectKnowledgeV1`.

Chronos deliberately retains its local type for now. Its zero-runtime-dependency
and single-package archive baseline is a release invariant; importing an unpublished
workspace path crate would weaken that profile. Executable correspondence provides
safe compatibility without pretending packaging requirements match.

## Rejected migrations

| Candidate | Decision | Reason |
| --- | --- | --- |
| Bounded data | Not qualified | Chronos `BoundedWord` has explicit absorbing overflow and allocation-capacity values; Keystone wrappers return construction/refusal errors. |
| Codec | Not qualified | Chronos checkpoint decoding owns schema versions, canonical fixtures, migration, time representation, and engine-specific ceilings absent from the generic cursor. |
| Checkpoint storage | Not qualified | Chronos binds snapshot revisions, acknowledgment strength, verified recovery attribution, retention scanning, and a qualified directory protocol. Keystone currently has a weaker mechanism-only CAS. |
| Directory adapter | Explicitly unavailable | Keystone's directory crate is a scaffold; Chronos has host/filesystem-scoped crash evidence that must not be generalized. |
| Observation | Not qualified | Chronos records a closed operation/disposition vocabulary and timing semantics; Keystone's generic recorder establishes only bounded dropping. |
| Clock/testkit | Not qualified | Chronos models `SystemTime`, duration arithmetic, negative epoch persistence, and monotone engine time; Keystone's `u64` logical clock is narrower. |
| Lifecycle | Not qualified | Chronos task/lease/grant lifecycle is domain authority, not a generic obligation ledger. |
| Redaction | Pattern compatible only | Chronos debug surfaces are domain-specific and already tested; wrapping them would change public types without additional evidence. |

## Adoption rule

Future migration requires a new row, explicit semantic comparison, executable
cross-project scenarios, and green gates in Keystone and every affected consumer.
Similarity of names or implementation shape is insufficient.

