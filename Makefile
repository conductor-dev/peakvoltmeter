.PHONY = run clean format lint

run:
	cargo run --release

clean:
	rm -rf target

format:
	cargo fmt

lint:
	cargo clippy -- -D warnings
