BUILD_DIR := build

# Remove all build outputs and intermediate files.
clean:
	@ rm -rf $(BUILD_DIR)


benchmark: grotsky grotsky-rs
	python tool/benchmark.py $(BUILD_DIR)/grotsky $(BUILD_DIR)/grotsky-rs loop 

test_grotsky:
	@ python tool/test.py go

test_grotsky-rs:
	@ python tool/test.py rust

test_all:
	@ python tool/test.py all

grotsky:
	@ go build cmd/grotsky/main.go
	@ mv main $(BUILD_DIR)/grotsky

grotsky-rs:
	@ cd rewrite_in_rust && cargo build --release && cp target/release/grotsky-rs ../$(BUILD_DIR)/grotsky-rs
