#!/bin/bash

cargo build --release

# we take 3 args, start block, end block, target address

if [ "$#" -ne 3 ]; then
    echo "Usage: $0 <start block> <end block> <target address>"
    exit 1
fi

# make the target lowercase
target=$(echo $3 | tr '[:upper:]' '[:lower:]')

export target

seq $1 $2 | parallel -j 100 --resume-failed --retries 3 --joblog joblog.txt \
    'block_hex=$(printf "0x%x" {});
    curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"debug_traceBlockByNumber\",\"params\":[\"$block_hex\", {\"tracer\":\"callTracer\"}],\"id\":1}" \
        http://192.168.1.40:8545 \
    | ./target/release/trace-history $target' \
    | tee -a results.txt