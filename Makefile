.PHONY: test

test:
	cargo test -- --nocapture
	cargo test --features="experimental" -- --nocapture
	cargo test --features="scoped_dictionary" -- --nocapture
	cargo run --bin pumpkindb-doctests
	cargo run --bin pumpkindb-doctests --features="experimental"
