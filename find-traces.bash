#!/bin/bash

set -e

cargo build --release

mkdir -p data

# we take at least 3 args: start block, end block, and one or more target addresses

if [ "$#" -lt 3 ]; then
    echo "Usage: $0 <start block> <end block> <target address1> [<target address2> ...]"
    exit 1
fi

start_block=$1
end_block=$2
shift 2

# make all targets lowercase
targets=$(echo "$@" | tr '[:upper:]' '[:lower:]')

export targets

seq $start_block $end_block | parallel -j 16 --resume-failed --retries 3 --joblog data/find-traces-joblog.txt \
    'block_hex=$(printf "0x%x" {});
    curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"debug_traceBlockByNumber\",\"params\":[\"$block_hex\", {\"tracer\":\"callTracer\"}],\"id\":1}" \
        http://127.0.0.1:8545 \
    | ./target/release/filter-block-trace $targets' \
    >> data/find-traces-results.txt