.PHONY: test doc

test:
	cargo test -- --nocapture
	cargo test --features="experimental" -- --nocapture
	cargo test --features="scoped_dictionary" -- --nocapture
	cargo test --features="static_module_dispatch" -- --nocapture
	cargo run --bin pumpkindb-doctests
	cargo run --bin pumpkindb-doctests --features="experimental"
	cargo run --bin pumpkindb-doctests --features="static_module_dispatch"

doc:
	cargo doc --lib
