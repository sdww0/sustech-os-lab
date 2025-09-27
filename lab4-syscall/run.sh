#! /bin/bash

mkdir -p target
riscv64-unknown-elf-gcc -static -O2 -mabi=lp64 -nostdlib ./user/hello.S -o target/hello
riscv64-unknown-elf-gcc -static -O2 -mabi=lp64 -nostdlib ./user/ebreak.S -o target/ebreak
cargo osdk run --target-arch=riscv64
