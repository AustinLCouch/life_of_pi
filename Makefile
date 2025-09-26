.PHONY: build pi dev clean check fmt clippy test

# Build for local development (macOS)
dev:
	cargo build --target aarch64-apple-darwin

# Cross-compile for Raspberry Pi
pi:
	cargo build --release --target aarch64-unknown-linux-gnu

# Run locally for development (with mock data)
run:
	cargo run --target aarch64-apple-darwin

# Quality checks
check:
	cargo check --all-targets
	cargo fmt -- --check
	cargo clippy --all-targets -- -D warnings

# Format code
fmt:
	cargo fmt

# Linting
clippy:
	cargo clippy --all-targets -- -D warnings

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean

# Deploy to Raspberry Pi (update IP address)
deploy: pi
	scp target/aarch64-unknown-linux-gnu/release/life_of_pi pi@YOUR_PI_IP:/home/pi/life_of_pi

# Install cross compilation tools (run once)
install-cross:
	brew install aarch64-elf-gcc
	rustup target add aarch64-unknown-linux-gnu