#!/bin/bash

set -e

sudo apt-get update
sudo apt-get install -y --no-install-recommends libc6-riscv64-cross
sudo apt-get install -y --no-install-recommends binutils-riscv64-linux-gnu gcc-riscv64-linux-gnu gdb-multiarch
sudo apt-get install -y --no-install-recommends binutils-riscv64-unknown-elf gcc-riscv64-unknown-elf
