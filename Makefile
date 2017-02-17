.PHONY: test

test:
	cargo test --features=travis -- --nocapture
	cargo test --features="experimental" -- --nocapture
	cargo test --features="scoped_dictionary" -- --nocapture
