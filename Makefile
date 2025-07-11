BUILD_DIR := build

# Remove all build outputs and intermediate files.
clean:
	@ rm -rf $(BUILD_DIR)


benchmark_loop: grotsky grotsky-rs
	python tool/benchmark.py $(BUILD_DIR)/grotsky $(BUILD_DIR)/grotsky-rs loop 

benchmark_fib: grotsky grotsky-rs
	python tool/benchmark.py $(BUILD_DIR)/grotsky $(BUILD_DIR)/grotsky-rs fib

benchmark_objects: grotsky grotsky-rs
	python tool/benchmark.py $(BUILD_DIR)/grotsky $(BUILD_DIR)/grotsky-rs objects

test_grotsky: grotsky
	@ cd archive && go test -v ./... -interpreter Go

test_grotsky-rs: grotsky-rs
	@ cd archive && RUST_BACKTRACE=1 go test -v ./... -interpreter Rust -failfast

test_integration: grotsky-rs
	@ python test/integration/blog.py || exit 1

grotsky:
	@ mkdir -p $(BUILD_DIR)
	@ cd archive && go build cmd/grotsky/main.go && mv main ../$(BUILD_DIR)/grotsky

grotsky-rs:
	@ mkdir -p $(BUILD_DIR)
	@ cargo build --release
	@ cp target/release/grotsky-rs build/
