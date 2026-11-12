# Bounded replay register

`keystone-replay` extracts the exact common mechanism used by Vellum and Agora:
a finite volatile relation from `(key, exact input)` to retained output.

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

Agora additionally validates command shape between new-key classification and its
capacity refusal. Vellum performs domain validation during interpretation after the
generic new decision. That ordering is domain policy and remains outside this crate;
their replay relation and overload laws are otherwise identical.
