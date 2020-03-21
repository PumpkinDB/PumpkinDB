.PHONY: test doc

test:
	cargo build --all --verbose
	cargo test --all -- --nocapture
	# Option --features not accepted any more in a virtual root
	#error: --features is not allowed in the root of a virtual workspace
    #note: while this was previously accepted, it didn't actually do anything
	#cargo test --all --features="experimental" -- --nocapture
	#cargo test --all --features="scoped_dictionary" -- --nocapture
	./target/debug/pumpkindb-doctests

doc:
	cargo doc --all --lib
