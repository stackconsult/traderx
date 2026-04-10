# TraderX Installation Guide

## System Requirements

### Hardware
- **CPU**: Intel Xeon Gold 6338N or equivalent (32+ cores recommended)
- **Memory**: 256GB DDR4 (minimum 64GB)
- **Network**: Solarflare/NVIDIA X2522 or similar kernel bypass NIC
- **Storage**: NVMe SSD for low-latency logging

### Software
- **OS**: Linux 6.5+ (Ubuntu 22.04 LTS recommended)
- **Kernel**: Must support eBPF/XDP and DPDK
- **PTP**: Hardware timestamping support for sub-microsecond sync

## Installation Steps

### 1. Install System Dependencies

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y \
    build-essential \
    cmake \
    pkg-config \
    libnuma-dev \
    libpcap-dev \
    python3.11 \
    python3-pip \
    redis-server \
    git

# Enable huge pages for DPDK
sudo sysctl vm.nr_hugepages=1024
echo 'vm.nr_hugepages=1024' | sudo tee -a /etc/sysctl.conf
```

### 2. Install Rust Toolchain

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup update stable
rustup component add rustfmt clippy
```

### 3. Install Python Dependencies

```bash
# Create virtual environment
python3 -m venv venv
source venv/bin/activate

# Upgrade pip
pip install --upgrade pip

# Install requirements
cd packages/execution-adapters
pip install -r requirements.txt
cd ../..

# Install additional HFT packages
pip install databento==0.42.0
pip install polygon==1.0.0
pip install quickfix==1.15.0
```

### 4. Install DPDK (Optional for <500ns latency)

```bash
# Download and build DPDK
cd /opt
sudo git clone http://dpdk.org/git/dpdk/dpdk.git
cd dpdk
git checkout v22.11

# Build DPDK
export RTE_SDK=/opt/dpdk
export RTE_TARGET=x86_64-native-linuxapp-gcc
make install T=x86_64-native-linuxapp-gcc -j$(nproc)

# Load kernel modules
sudo modprobe uio
sudo modprobe vfio-pci
```

### 5. Build TraderX

```bash
# Build core components
cd packages/oms-engine
cargo build --release

# Build with DPDK support (optional)
cd ../dealing-desk/ebpf-router
cargo build --release --features dpdk

# Build main HFT system
cd ../../hft-system
cargo build --release
```

### 6. Start Required Services

```bash
# Start Redis for journaling
sudo systemctl start redis-server
sudo systemctl enable redis-server

# Start Aeron media driver (optional, for ultra-low latency)
# Download from https://github.com/real-logic/aeron
# aeron-driver-1.40.0-linux64/bin/aeron-driver
```

## Configuration

### Environment Variables

```bash
# Add to ~/.bashrc
export TRADERX_CONFIG=/path/to/config
export RTE_SDK=/opt/dpdk  # If using DPDK
export AERON_DIR=/path/to/aeron  # If using Aeron
```

### Network Configuration

```bash
# Disable IRQ balancing for kernel bypass NIC
sudo apt install irqbalance
sudo systemctl stop irqbalance
sudo systemctl disable irqbalance

# Set NIC to userspace driver (DPDK)
sudo dpdk-devbind.py --bind=vfio-pci 0000:3b:00.0
```

## Performance Tuning

### CPU Isolation

```bash
# Isolate CPU cores for trading
echo 'isolcpus=2-31' | sudo tee -a /etc/default/grub
sudo update-grub
sudo reboot
```

### Network Optimization

```bash
# Increase network buffer sizes
echo 'net.core.rmem_max = 134217728' | sudo tee -a /etc/sysctl.conf
echo 'net.core.wmem_max = 134217728' | sudo tee -a /etc/sysctl.conf
echo 'net.core.netdev_max_backlog = 5000' | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

### PTP Time Sync

```bash
# Install linuxptp
sudo apt install linuxptp

# Configure PTP
sudo systemctl start ptp4l
sudo systemctl enable ptp4l
```

## Verification

### 1. Test Python Dependencies

```bash
cd /path/to/traderx
python3 test_dependencies.py
```

### 2. Test Rust Compilation

```bash
cd packages/oms-engine
cargo test --release

cd ../dealing-desk/ebpf-router
cargo test --release --features dpdk
```

### 3. Run Benchmarks

```bash
# OMS benchmarks
cd packages/oms-engine
cargo run --release --bin oms_benchmark

# Full system test
cd apps/trading_engine
cargo run --release --benchmark
```

## Expected Performance

| Component | Latency | Throughput |
|-----------|---------|------------|
| OMS Order Submission | <1μs | 2.5M ops/sec |
| eBPF/XDP Router | 1-2μs | 10M packets/sec |
| DPDK Router | <500ns | 20M packets/sec |
| Aeron Journal | 18μs | 5M events/sec |
| Redis Journal | 50-100μs | 100K events/sec |

## Troubleshooting

### Common Issues

1. **DPDK not found**: Ensure DPDK is built and RTE_SDK is set
2. **Permission denied**: Use sudo for network operations
3. **Huge pages**: Verify /proc/meminfo shows huge pages
4. **Aeron driver**: Media driver must be running before Aeron apps

### Debug Mode

```bash
# Enable debug logging
export RUST_LOG=debug
export TRADERX_DEBUG=1

# Run with strace for system calls
strace -c cargo run --release
```

## Production Deployment

1. **Co-location**: Deploy in exchange data center
2. **Network**: Direct fiber connection to exchange
3. **Hardware**: Use kernel bypass NICs
4. **Monitoring**: STAC-T1 benchmark compliance
5. **Backup**: Redundant systems with automatic failover

## Support

For issues:
1. Check logs in `/var/log/traderx/`
2. Review benchmark results in `proofs/`
3. Consult STAC compliance reports
4. Contact support with system specs
