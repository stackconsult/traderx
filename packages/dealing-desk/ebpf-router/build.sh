#!/bin/bash

# Build script for TraderX eBPF/XDP Router
# This builds the actual production eBPF program

set -e

echo "Building TraderX eBPF/XDP Router..."

# Check prerequisites
if ! command -v cargo &> /dev/null; then
    echo "ERROR: cargo not found. Install Rust first."
    exit 1
fi

if ! command -v clang &> /dev/null; then
    echo "ERROR: clang not found. Install clang for eBPF compilation."
    exit 1
fi

# Check kernel headers
if [ ! -d "/usr/src/linux-headers-$(uname -r)" ]; then
    echo "WARNING: Kernel headers not found. Install: apt-get install linux-headers-$(uname -r)"
fi

# Build eBPF program
echo "Building eBPF program..."
cd ebpf
cargo build-bpf --release --target bpfel-unknown-none

# Build userspace loader
echo "Building userspace loader..."
cd ..
cargo build --release

# Verify eBPF object was created
if [ ! -f "ebpf/target/bpfel-unknown-none/release/traderx-ebpf-router" ]; then
    echo "ERROR: eBPF object file not found"
    exit 1
fi

# Check eBPF program size
SIZE=$(stat -c%s "ebpf/target/bpfel-unknown-none/release/traderx-ebpf-router")
echo "eBPF program size: $SIZE bytes"

if [ $SIZE -gt 4096 ]; then
    echo "WARNING: eBPF program exceeds 4KB limit ($SIZE bytes)"
fi

# Verify with bpftool if available
if command -v bpftool &> /dev/null; then
    echo "Verifying eBPF program with bpftool..."
    bpftool prog dump file "ebpf/target/bpfel-unknown-none/release/traderx-ebpf-router"
fi

echo "Build complete!"
echo "To run: sudo ./target/release/traderx-ebpf-router --iface eth0"
