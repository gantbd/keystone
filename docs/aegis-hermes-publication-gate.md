# Aegis and Hermes durable publication gate

Status: qualified shared mechanism as of 2026-11-10.

## Matching contract

Aegis and Hermes each serialize one live owner over a `CheckpointStore`. Both
prepare a complete domain candidate before the store call and retain one exact
last-confirmed storage revision. Both release the candidate state and operation
result only after receiving a durable receipt for the proposed next revision.

The shared post-offer outcomes are:

| Storage evidence | Gate result | Live owner |
| --- | --- | --- |
| Exact revision and `Durable` strength | confirmed | may publish candidate |
| Volatile/weak receipt | invalid receipt | fenced |
| Wrong revision | invalid receipt | fenced |
| Store refusal | refused | fenced |
| Indeterminate outcome | indeterminate | fenced |
| Store error | store failure | fenced |

Revision exhaustion is different: it is detected before encoding is offered to
storage and leaves the gate unfenced at its last confirmed revision.

`keystone-storage::CommitGate` implements only that table. It does not call the
store, hold candidate state, encode a checkpoint, publish an outcome, retry,
recover, authenticate a caller or interpret a domain refusal.

## Consumer ownership retained

Aegis retains opaque grants, authorization decisions, epoch fencing, fixed-size
checkpoint encoding and the rule that reopening must durably advance the epoch
before use. Hermes retains streams, append/replay/conflict semantics, consumer
cursors, checkpoint budgets and the rule that replay and identity acknowledgment
do not write.

Both hosts retain their own public error vocabularies and content-free signals.
Their migrations map the shared gate result back into those existing types.

## Evidence

Keystone law tests cover initial and successive exact durable receipts, every
post-offer failure class, sticky fencing and non-fencing revision exhaustion.
Aegis's existing durable-host scenarios cover all four authority operations,
before/after-commit uncertainty, weak/wrong receipts, store refusal/error,
recovery fencing and pre-offer refusal identity. Hermes's scenarios cover
append and cursor publication, retained replay, before/after uncertainty,
weak/wrong receipts, recovery, and checkpoint-encoding refusal before offer.

From a workspace checkout containing the three sibling repositories,
`make qualify-aegis-hermes` executes both durable-host suites against the same
local `keystone-storage` source. This is deliberately separate from Keystone's
self-contained `make check`: a production Keystone checkout does not require
consumer repositories.

This qualifies the publish-after-receipt gate for these two serialized local
hosts. It does not qualify distributed consensus, multi-writer storage,
automatic retry, physical power-loss behavior, or either domain's recovery
policy as a generic Keystone facility.

