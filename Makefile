.PHONY: test doc

test:
	cargo build --all --verbose
	cargo test --all -- --nocapture
	cargo test --all --features="experimental" -- --nocapture
	cargo test --all --features="scoped_dictionary" -- --nocapture
	cargo test --all --features="static_module_dispatch" -- --nocapture
	./target/debug/pumpkindb-doctests

doc:
	cargo doc --all --lib
