# Contributing

Keystone contains small, reusable mechanisms shared by bounded engines. Keep
changes mechanism-focused: domain schemas, commands, refusals, migrations,
resource policy, and semantic store contracts belong in their owning engines.

Before changing behavior, read the relevant crate documentation and its linked
profile in `qualification-status.json`. A crate or API must not be described as
qualified unless the machine-checked classification and matching evidence are
updated together. Preserve explicit bounds, content-safe diagnostics, typed
uncertainty, and the nonclaims documented beside each mechanism.

Run the production workspace gate before submitting a change:

```sh
make check
```

Cross-project qualification is intentionally separate. Run the applicable
target when changing a shared consumer contract:

```sh
make qualify-aegis-hermes
make qualify-chronos-moira
make qualify-framing
make qualify-unix-listener
```

Keystone is source-distributed from GitHub and is not published to crates.io.
Do not add publication metadata, domain authority, or unrelated dependencies
without an explicit architecture and distribution decision.
