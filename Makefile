.PHONY: check check-boundaries fmt lint test doc qualify-chronos-moira

check: check-boundaries fmt lint test doc

check-boundaries:
	node scripts/check-project-boundaries.mjs

fmt:
	cargo fmt --all -- --check

lint:
	cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings

test:
	cargo test --workspace --all-targets --locked --offline

doc:
	cargo test --workspace --doc --locked --offline
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked --offline

qualify-chronos-moira:
	cargo test --manifest-path qualification/chronos-moira/Cargo.toml --all-targets --locked --offline

