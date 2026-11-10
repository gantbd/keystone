# Vellum codec qualification

Status: narrow consumer qualification completed 2026-11-09.

Vellum is the second concrete consumer for `keystone-codec`. The adopted
surface is deliberately small:

- `Decoder` and `DecodeLimits` bound complete input bytes, individual byte
  fields, and total decoded fields;
- `read_u8`, `read_u64_be`, and `read_bytes` provide deterministic canonical
  primitives;
- `encode_bytes` supplies the unsigned 32-bit big-endian byte-field form.

Vellum retains ownership of protocol versions, frame and variant tags, identity
validity, strict UTF-8, text limits, compatibility, disclosure, and closed
errors. Its encoder computes and reserves an exact bounded frame before calling
the Keystone byte primitive. No Vellum domain vocabulary enters Keystone.

Consumer evidence lives in
[`vellum-wire`](https://github.com/gantbd/vellum-sdk/tree/main/crates/vellum-wire).
Golden fixtures cover connection and semantic request/reply frames. Adversarial
tests cover every truncation point of a semantic request, trailing bytes,
frame/field/text ceilings, unknown versions and tags, invalid identities and
UTF-8, and every closed Vellum refusal/outcome variant. The Vellum dependency
gate permits `vellum-wire -> keystone-codec` while forbidding wire dependencies
on the engine, host, session, editor, or service.

This qualification establishes the cursor and byte-field primitives for the
bounded Vellum V1 codec. It does not establish generic golden-fixture tooling,
stream framing, daemon safety, cryptographic integrity, authentication,
filesystem durability, performance, or suitability for another component's
schema. Those require their own consumer evidence.
