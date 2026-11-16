# Bounded replay register

`keystone-replay` is the finite volatile relation adopted by Vellum from
`(key, exact input)` to retained output. Agora was compared because it exposes
the same four decisions, but it is not a consumer of this mechanism.

The four decisions are closed:

- `New`: the key is absent and capacity remains;
- `MatchingReplay`: the key and exact input match, returning the original output;
- `Conflict`: the key exists with different input;
- `Full`: the key is absent and capacity is exhausted.

Existing-key lookup always precedes the capacity check. A full register therefore
continues to distinguish replay from conflict. There is no eviction, expiry, retry,
clock, authority check, effect execution, persistence, or transport policy.

Construction fallibly preallocates the full entry capacity. The owning serialized
interpreter classifies before executing its domain operation and then records the
result with `record_new`; the second method rechecks the law and cannot allocate.
Explicit `remove_where` supports owner lifecycle cleanup such as client disconnect.
It is never called automatically under pressure.
Register `Debug` reports only length and capacity; keys, inputs, and outputs
remain consumer-owned and are not disclosed.

Vellum performs domain validation during interpretation after the generic new
decision and owns all request and reply vocabulary.

Agora retains separate registries. Its V1 service validates command shape before
mapping an absent key to capacity refusal. Its V2 receipts are part of canonical
checkpoint recovery, carry a mutable policy-selected live limit, and can be
replaced by durable non-reexecution tombstones. Agora's export registries also
require key-ordered checkpoint material. Those stores allocate incrementally;
this register fallibly preallocates its fixed capacity. The decisions correspond,
but allocation, persistence, ordering and lifecycle laws do not, so direct
adoption would erase meaningful guarantees rather than normalize a mechanism.
