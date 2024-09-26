#!/bin/bash

cargo build --release

# we take 3 args, start block, end block, target address

if [ "$#" -ne 3 ]; then
    echo "Usage: $0 <start block> <end block> <target address>"
    exit 1
fi

# make the target lowercase
target=$(echo $3 | tr '[:upper:]' '[:lower:]')

seq $1 $2 | parallel -j 45 --resume-failed --retries 3 --joblog joblog.txt ./target/release/trace-history {} $target | tee results.txt

