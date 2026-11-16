.PHONY: check check-boundaries fmt lint test doc qualify-aegis-hermes qualify-chronos-moira qualify-framing qualify-unix-listener

check: check-boundaries fmt lint test doc

check-boundaries:
	node scripts/check-project-boundaries.mjs

fmt:
	cargo fmt --all -- --check

lint:
	cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings

test:
	cargo test --workspace --all-targets --all-features --locked --offline

doc:
	cargo test --workspace --doc --all-features --locked --offline
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --locked --offline

qualify-aegis-hermes:
	cargo test --manifest-path ../aegis/Cargo.toml -p aegis-host --lib --test durable_behavior --locked --offline
	cargo test --manifest-path ../hermes/Cargo.toml -p hermes-host --lib --test durable_behavior --locked --offline

qualify-chronos-moira:
	cargo test --manifest-path qualification/chronos-moira/Cargo.toml --all-targets --locked --offline

qualify-framing:
	cargo test --manifest-path ../aegis/Cargo.toml -p aegis-local-daemon --lib --test connection_behavior --locked --offline
	cargo test --manifest-path ../hermes/Cargo.toml -p hermes-local-daemon --lib --test connection_behavior --locked --offline
	cargo test --manifest-path ../moira/Cargo.toml -p moira-local-daemon --lib --test connection_behavior --locked --offline

qualify-unix-listener:
	cargo test --manifest-path ../aegis/Cargo.toml -p aegis-local-daemon --test unix_listener --locked --offline
	cargo test --manifest-path ../hermes/Cargo.toml -p hermes-local-daemon --test unix_listener --locked --offline
	cargo test --manifest-path ../moira/Cargo.toml -p moira-local-daemon --test unix_listener --test saga_unix_listener --locked --offline

