BUILD_DIR := build

# Remove all build outputs and intermediate files.
clean:
	@ rm -rf $(BUILD_DIR)


benchmark_loop: grotsky grotsky-rs
	python3 tool/benchmark.py $(BUILD_DIR)/grotsky $(BUILD_DIR)/grotsky-rs loop 

benchmark_fib: grotsky grotsky-rs
	python3 tool/benchmark.py $(BUILD_DIR)/grotsky $(BUILD_DIR)/grotsky-rs fib

benchmark_objects: grotsky grotsky-rs
	python3 tool/benchmark.py $(BUILD_DIR)/grotsky $(BUILD_DIR)/grotsky-rs objects

test_grotsky: grotsky
	@ cd archive && go test -v ./... -interpreter Go

test_grotsky-rs: grotsky-rs
	@ cd archive && RUST_BACKTRACE=1 go test -v ./... -interpreter Rust -failfast

test_integration: grotsky-rs
	@ python3 test/integration/blog.py || exit 1

# Coverage target added
run_coverage_tests: grotsky-rs
	@ export LLVM_PROFILE_FILE="grotsky-cov-%p-%m.profraw" && \
	  ./build/grotsky-rs test/coverage_tests.gr && \
	  ./build/grotsky-rs test/expanded_coverage.gr

run_embed_test: grotsky-rs
	@ export LLVM_PROFILE_FILE="grotsky-embed-%p-%m.profraw" && \
	  python3 test/integration/embed_test.py

run_net_test: grotsky-rs
	@ export LLVM_PROFILE_FILE="grotsky-net-%p-%m.profraw" && \
	  python3 test/integration/net_test.py

coverage: clean
	@ cargo clean
	@ mkdir -p $(BUILD_DIR)
	@ export RUSTFLAGS="-C instrument-coverage" && \
	  cargo build --release
	@ cp target/release/grotsky-rs build/
	@ export LLVM_PROFILE_FILE="grotsky-%p-%m.profraw" && \
	  $(MAKE) test_grotsky-rs && \
	  $(MAKE) test_integration && \
	  $(MAKE) run_coverage_tests && \
	  $(MAKE) run_embed_test && \
	  $(MAKE) run_net_test
	@ echo "Collecting coverage data..."
	@ grcov . --binary-path ./target/release/ -s . -t lcov --branch --ignore-not-existing --ignore "target/*" --ignore "archive/*" --ignore "test/*" -o lcov.info
	@ if [ -f lcov.info ]; then \
		echo "Coverage report generated at lcov.info"; \
		echo "Run 'python3 tool/analyze_coverage.py' to analyze it"; \
	else \
		echo "Error: Failed to generate lcov.info. Check that .profraw files exist."; \
		exit 1; \
	fi

grotsky:
	@ mkdir -p $(BUILD_DIR)
	@ cd archive && go build cmd/grotsky/main.go && mv main ../$(BUILD_DIR)/grotsky

grotsky-rs:
	@ mkdir -p $(BUILD_DIR)
	@ cargo build --release
	@ cp target/release/grotsky-rs build/
