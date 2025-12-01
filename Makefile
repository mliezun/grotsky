BUILD_DIR := build

# Remove all build outputs and intermediate files.
clean:
	@ rm -rf $(BUILD_DIR)


benchmark_loop: grotsky grotsky-rs
	python3 tool/benchmark.py $(BUILD_DIR)/grotsky $(BUILD_DIR)/grotsky-rs loop 100

benchmark_fib: grotsky grotsky-rs
	python3 tool/benchmark.py $(BUILD_DIR)/grotsky $(BUILD_DIR)/grotsky-rs fib 100

benchmark_objects: grotsky grotsky-rs
	python3 tool/benchmark.py $(BUILD_DIR)/grotsky $(BUILD_DIR)/grotsky-rs objects 100

profile_loop: grotsky
	@ cargo build --release --features profile
	@ cp target/release/grotsky-rs build/grotsky-rs
	@ GROTSKY_PROFILE=1 python3 tool/benchmark.py $(BUILD_DIR)/grotsky-rs loop

profile_fib: grotsky
	@ cargo build --release --features profile
	@ cp target/release/grotsky-rs build/grotsky-rs
	@ GROTSKY_PROFILE=1 python3 tool/benchmark.py $(BUILD_DIR)/grotsky-rs fib

profile_objects: grotsky
	@ cargo build --release --features profile
	@ cp target/release/grotsky-rs build/grotsky-rs
	@ GROTSKY_PROFILE=1 python3 tool/benchmark.py $(BUILD_DIR)/grotsky-rs objects

profile_integration:
	@ cargo build --release --features profile
	@ cp target/release/grotsky-rs build/grotsky-rs
	@ GROTSKY_PROFILE=1 GROTSKY_PROFILE_OUTPUT=$(shell pwd) python3 test/integration/blog.py

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
	  ./build/grotsky-rs test/expanded_coverage.gr && \
	  ./build/grotsky-rs test/comprehensive_coverage.gr && \
	  ./build/grotsky-rs test/coverage_gap_closing.gr

run_embed_test: grotsky-rs
	@ export LLVM_PROFILE_FILE="grotsky-embed-%p-%m.profraw" && \
	  python3 test/integration/embed_test.py

run_net_test: grotsky-rs
	@ export LLVM_PROFILE_FILE="grotsky-net-%p-%m.profraw" && \
	  python3 test/integration/net_test.py

coverage: clean
	@ rm -f *.profraw archive/*.profraw
	@ cargo clean
	@ mkdir -p $(BUILD_DIR)
	# Build instrumented binary for integration tests
	@ export RUSTFLAGS="-C instrument-coverage" && \
	  export LLVM_PROFILE_FILE="grotsky-%p-%m.profraw" && \
	  export DEBUG_BUILD=1 && \
	  cargo build && cp target/debug/grotsky-rs build/
	# Run all tests
	@ export RUSTFLAGS="-C instrument-coverage" && \
	  export LLVM_PROFILE_FILE="grotsky-%p-%m.profraw" && \
	  export DEBUG_BUILD=1 && \
	  $(MAKE) test_grotsky-rs && \
	  $(MAKE) test_integration && \
	  $(MAKE) run_coverage_tests && \
	  $(MAKE) run_embed_test && \
	  $(MAKE) run_net_test
	# Generate report using grcov
	@ echo "Collecting coverage data..."
	@ grcov . --binary-path target/debug/ -s . -t lcov --branch --ignore-not-existing --ignore "/*" --ignore "test/**" --ignore "archive/**" --ignore "tool/**" --ignore "examples/**" -o lcov.info

grotsky:
	@ mkdir -p $(BUILD_DIR)
	@ cd archive && go build cmd/grotsky/main.go && mv main ../$(BUILD_DIR)/grotsky

grotsky-rs:
	@ mkdir -p $(BUILD_DIR)
	@ if [ -z "$(DEBUG_BUILD)" ]; then \
		cargo build --release && cp target/release/grotsky-rs build/; \
	else \
		cargo build && cp target/debug/grotsky-rs build/; \
	fi
