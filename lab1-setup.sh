#!/bin/bash

set -e

sudo apt-get install -y tree vim curl build-essential ca-certificates python3-pip python-is-python3 wget gcc make pkg-config gdb grub-efi-amd64 grub2-common libpixman-1-dev xorriso openssh-server net-tools

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
cargo install cargo-binutils
