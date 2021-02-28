build: build-lending build-margin

build-lending:
	cd token-lending/program; cargo build-bpf

build-margin:
	anchor build

clean:
	@echo "Cleaning local packages..."
	@cargo clean -p margin-account
	@cargo clean -p spl-token-lending
	@echo "Done cleaning."

lint: clean
	cargo fmt --all
	cargo clippy --all-features -- -D warnings

test: test-margin

test-lending:
	cd token-lending/program; cargo test-bpf

# Needs to build lending program to test full functionality
test-margin: build-lending
	anchor test

.PHONY: clean build lint
