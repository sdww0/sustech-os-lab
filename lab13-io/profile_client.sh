#! /bin/bash

sudo apt-get install gdb-multiarch

INTEVAL=0.1
SAMPLES=200

# Prepare

rm -f gdb_perf.log
rm -rf FlameGraph
git clone https://github.com/brendangregg/FlameGraph.git

# Run gdb-multiarch "SAMPLES" times with interval "INTEVAL" seconds to collect profiling data

for i in $(seq 1 $SAMPLES)
do
    echo "Collecting sample $i / $SAMPLES ..."
    gdb-multiarch -batch -ex "set pagination 0" \
        -ex "file target/riscv64imac-unknown-none-elf/release/lab13-io-osdk-bin" \
        -ex "target remote :1234" \
        -ex "bt -frame-arguments presence -frame-info short-location" \
        -ex "continue" >> gdb_perf.log &
    
    sleep $INTEVAL
    kill -SIGINT $(pgrep gdb-multiarch)
done

# Generate the flamegraph

python gdb_perf.py
./FlameGraph/flamegraph.pl ./out.folded > kernel.svg

