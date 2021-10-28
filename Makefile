fmt:
	cargo fmt --all
clippy:
	cargo +nightly clippy -- -D warnings -W clippy::cognitive_complexity
build:
	cargo build
release:
	cargo build --release
clean:
	cargo clean