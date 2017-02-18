.PHONY: test

test:
	cargo test -- --nocapture
	cargo test --features="experimental" -- --nocapture
	cargo test --features="scoped_dictionary" -- --nocapture
