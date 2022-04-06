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

	# Progress check
	cd scripts/word_count && cargo fmt && cargo run

# TODO: clean code_snippet binaries
clean:
	mdbook clean
	cd scripts/word_count && cargo clean

site:
	rm -rf docs/
	mdbook build
	mv book/ docs/
	cp CNAME docs/