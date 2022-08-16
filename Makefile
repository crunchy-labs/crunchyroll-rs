test:
	cargo test --no-fail-fast -- --test-threads=1

test-strict:
	cargo test --features __test_strict --no-fail-fast -- --test-threads=1
