# Observation accumulators

Status: counter law adopted by Aegis and Hermes; clamping duration law adopted
by Aegis. Hermes's non-clamping duration law remains local.

`keystone-observe` supplies two non-authoritative accumulators for fixed-cardinality
instrumentation: `SaturatingCounter` for `u64` counts and `DurationAccumulator`
for `std::time::Duration` totals. They read no clock, allocate no storage and
carry no domain operation vocabulary.

## Laws

For either accumulator, with maximum value `MAX`:

1. The initial value is zero with no saturation evidence.
2. Adding zero before saturation is an exact identity transition.
3. If `value + offered <= MAX`, the new value is the exact sum and the result is
   `Accumulation::Exact`.
4. Reaching `MAX` exactly does not set the saturation bit.
5. If `value + offered > MAX`, the value becomes `MAX`, the saturation bit is
   set and the result is `Accumulation::SaturatedNow`.
6. Once saturated, every later offer leaves the state unchanged at `MAX` and
   returns `Accumulation::AlreadySaturated`, including a later zero offer.
7. Saturation is evidence that the exact total is no longer represented. It is
   never evidence about the success, duration or cardinality of a domain
   operation.

Consumers own counter names, observation enablement, reset/snapshot policy and
whether an overflow should trigger a domain-specific alert. No merge operation
is supplied: combining already-saturated observations requires consumer-owned
provenance and policy.

The executable unit tests cover every transition above for both value types.
Aegis and Hermes retain their public signal structures and operation vocabulary;
their durable-host tests prove observation remains non-authoritative. Aegis maps
both helpers directly. Hermes maps only the counter helper because its duration
overflow preserves the previous exact value instead of clamping to `MAX`; its
host test fixes that difference with a non-maximum overflow case. These
adoptions do not qualify any engine's complete instrumentation contract.
