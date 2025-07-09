build-musl:
	@make build-podman
	@echo "Building for musl..."
	$(eval CONTAINER_ID := $(shell podman create rust-musl-builder))

	@mkdir -p target/x86_64-unknown-linux-musl/interactive/en

	podman start -a $(CONTAINER_ID)
	podman cp $(CONTAINER_ID):/app/target/x86_64-unknown-linux-musl target/x86_64-unknown-linux-musl/
	podman rm $(CONTAINER_ID)

test:
	@echo "Running tests..."
	@cargo test

run:
	@echo "Running..."
	@cargo run

clean:
	@echo "Cleaning..."
	@cargo clean

build-podman:
	@echo "Building for podman..."
	podman build -t rust-musl-builder .

publish:
	@echo "Publishing..."
	cargo publish
	@mkdir -p target/publish
	@cp target/release/mcwr target/publish/mcwr
	@cp target/interactive/en/release/mcwr target/publish/mcwr-en
	@cp target/interactive/es/release/mcwr target/publish/mcwr-es
	@cp target/x86_64-unknown-linux-musl/release/mcwr target/publish/mcwr-musl
	@cp target/x86_64-unknown-linux-musl/interactive/en/release/mcwr target/publish/mcwr-en-musl
	@cp target/x86_64-unknown-linux-musl/interactive/es/release/mcwr target/publish/mcwr-es-musl
	gh release create "v0.1.8" ./target/publish/*
	@rm -rf target/publish

build-normal:
	@echo "Building for normal..."

	@mkdir -p target/interactive/en
	@mkdir -p target/interactive/es

	@cargo build --release
	@cargo build --target-dir target/interactive/en --features "interactive"
	@cargo build --target-dir target/interactive/es --features "interactive es"

build:
	@echo "Building..."
	@cargo build --release
	@make build-musl
