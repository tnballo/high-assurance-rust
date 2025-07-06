serve:
	mdbook serve

read:
	mdbook serve --open

# TODO: clean up command duplication, this is gross
check:
	# Book
	mdbook test

	# Code snippets
	cd code_snippets/chp2/crypto_tool && cargo fmt && cargo test && cargo test --all-features
	cd code_snippets/chp3/rc4 && cargo fmt && RUSTFLAGS=-Awarnings cargo test
	cd code_snippets/chp3/proc && cargo fmt && RUSTFLAGS=-Awarnings cargo test
	cd code_snippets/chp3/proc_2 && cargo fmt && RUSTFLAGS=-Awarnings cargo test
	cd code_snippets/chp3/prime_test && cargo test
	cd code_snippets/chp4/greeting && cargo fmt && cargo test
	cd code_snippets/chp4/stack_example && cargo fmt
	cd code_snippets/chp4/stack_example_iter && cargo fmt
	cd code_snippets/chp14/tactical_trust && cargo fmt && cargo test
	cd code_snippets/chp14/tt_test_crates/test_crypto_duplicates && cargo fmt && cargo test
	cd code_snippets/chp14/tt_test_crates/test_disallowed_crypto_publishers && cargo fmt && cargo test

	# Metrics and linting
	cd internal_tools/har_analyze && cargo fmt && cargo test && cargo clippy && cargo run -- --metrics --lint --log-warn

# TODO: clean code_snippet binaries
clean:
	mdbook clean
	cd internal_tools/har_analyze && cargo clean

site:
	cd internal_tools/har_analyze && cargo run -- --lint --update
	rm -rf docs/
	mdbook build
	mv book/ docs/
	cp CNAME docs/
