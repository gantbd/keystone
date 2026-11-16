# Cargo license declaration baseline

`scripts/check-cargo-license-baseline.mjs` is a dev-only assurance mechanism for
Cargo workspaces. It consumes complete `cargo metadata` JSON on standard input
and compares every package's exact name, version and declared SPDX expression to
consumer-owned `license|...` records.

The tool rejects missing declarations, added/removed packages and expression
drift. Consumers own their accepted baseline and legal review; Keystone defines
no license allowlist and does not claim that metadata proves source contents,
license compatibility or current advisory status. It is not a runtime dependency.

Aegis, Hermes and Moira independently run the tool from their locked offline
graphs, satisfying the admission requirement for multiple real consumers while
keeping release policy in each owning repository.
