.PHONY: run build clean

build:
	cargo build --release

clean:
	cargo clean

run:
	cargo run --release