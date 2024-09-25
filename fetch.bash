#!/bin/bash

cargo build --release

# we take 3 args, start block, end block, target address

if [ "$#" -ne 3 ]; then
    echo "Usage: $0 <start block> <end block> <target address>"
    exit 1
fi

seq $1 $2 | parallel -j 32 --resume-failed --retries 3 --joblog joblog.txt ./target/release/mantle-data-collection-fast {} $3 | tee results.txt

