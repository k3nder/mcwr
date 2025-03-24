build-musl:
	@echo "Building for musl..."
	$(eval CONTAINER_ID := $(shell podman create rust-musl-builder))
	podman start -a $(CONTAINER_ID)
	podman cp $(CONTAINER_ID):/app/target/x86_64-unknown-linux-musl/ target/
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
	gh release create $(shell git describe --tags --abbrev=0) target/release/mcwr target/release/mcwr-universal

build-normal:
	@echo "Building for normal..."
	@cargo build --release

build:
	@echo "Building..."
	@cargo build --release
	@make build-musl
