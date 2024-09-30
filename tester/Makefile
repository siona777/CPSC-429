# Makefile for compiling and running a Rust project

# Define the name of the Rust project
PROJECT_NAME = tester

# Path to the Rust binary after building in release mode
TARGET_PATH = target/release/$(PROJECT_NAME)

all: build

# Build the Rust project in release mode
build:
	cargo build --release

# Run the Rust project with sudo
run: build
	sudo insmod rust_mymem.ko
	sudo $(TARGET_PATH)
	sudo rmmod rust_mymem.ko

# Clean the project - remove target directory
clean:
	cargo clean

# Create documentation for the project
doc:
	cargo doc --open

.PHONY: all build run clean doc
