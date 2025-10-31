#! /bin/bash

num_runs=100
log_file=qemu.log

for i in $(seq 1 $num_runs); do
    ./a.out >"$log_file"
    # Check the log file contains the expected output
    if ! grep -q "1000000" "$log_file"; then
        echo "Run $i: Failure"
        cat "$log_file"
        exit 1
    fi
done

echo "All runs completed."
