#!/bin/bash

sudo apt-get install -y --no-install-recommends libusb-dev libglib2.0-dev libpixman-1-dev meson ninja-build libgcrypt-dev libslirp-dev libevent-dev ovmf
pip3 install yq tomli

# Install qemu
wget -O qemu.tar.xz https://download.qemu.org/qemu-8.2.1.tar.xz
mkdir qemu
tar -xf qemu.tar.xz -C qemu --strip-components=1
cd qemu
./configure --target-list=riscv64-softmmu,x86_64-softmmu --prefix=/usr/local/qemu --enable-slirp
make -j$(nproc)
sudo make install
echo 'export PATH=$PATH:/usr/local/qemu/bin' >>$HOME/.bashrc

cd ..
rm -rf qemu qemu.tar.xz

# Install OSDK
sudo apt-get install -y --no-install-recommends mtools

cargo install cargo-osdk --version 0.16.1
