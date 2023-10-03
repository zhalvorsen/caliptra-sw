# Setup FPGA

## Connect Serial

```bash
picocom /dev/ttyUSB1 --baud 115200 --imap lfcrlf
```

## Initial One-time Setup

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
sudo apt update
sudo apt install libssl-dev
git clone https://github.com/chipsalliance/caliptra-sw.git
cd caliptra-sw
git remote add zhalvorsen https://github.com/zhalvorsen/caliptra-sw.git
git fetch --all
git checkout zhalvorsen/fpga-build
git submodule update --init
cargo build
```

## Go to root

```bash
sudo bash
```

## Root Commands

```bash
echo 321 > /sys/class/gpio/export
echo out > /sys/class/gpio/gpio321/direction

cd ~ubuntu/caliptra-sw/hw-latest/fpga/caliptra_build
fpgautil -b caliptra_fpga.bin -f Full -n Full

cd ~ubuntu/caliptra-sw/hw-latest/fpga/rom_backdoor
make
insmod rom_backdoor.ko

cd ~ubuntu/caliptra-sw/hw-latest/fpga/io_module
make
insmod io_module.ko
chmod 666 /dev/uio4

echo 20000000 > /sys/bus/platform/drivers/xilinx_fclk/fclk0/set_rate
```
