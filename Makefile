BUILD_DIR := build

# Remove all build outputs and intermediate files.
clean:
	@ rm -rf $(BUILD_DIR)


benchmark: grotsky grotsky-rs
	python tool/benchmark.py $(BUILD_DIR)/grotsky $(BUILD_DIR)/grotsky-rs loop 

test_grotsky: grotsky
	@ go test -v ./... -interpreter Go

test_grotsky-rs: grotsky-rs
	@ RUST_BACKTRACE=1 go test -v ./... -interpreter Rust -failfast

test:
	@ go test -v ./...

grotsky:
	@ go build cmd/grotsky/main.go
	@ mv main $(BUILD_DIR)/grotsky

grotsky-rs:
	@ cd rewrite_in_rust && cargo build --release && cp target/release/grotsky-rs ../$(BUILD_DIR)/grotsky-rs