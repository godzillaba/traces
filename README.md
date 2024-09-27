# Traces

Use this tool to find and analyze traces that touch certain target accounts.

## Usage

### 1. Gather Traces

To gather all the traces that touch a target contract, use

```bash
./find-traces.bash <start block> <end block> <target address1> [<target address2> ...]
```

Results will land in `data/find-traces-results.txt`, with each transaction trace object on its own line.

This command can be safely stopped and will pick up where it left off when restarted. To completely restart, first remove `data/find-traces-joblog.txt`

### 2. Parse traces for a certain target

This will print a csv with call type, selector, and from
```bash
cargo run --bin parse-traces -- <target>
```

To exclude top level calls and pretty print selectors, use
```bash
cargo run --bin parse-traces -- <target> | grep -v toplevel | ./pretty-selectors
```

To copy to clipboard to put into a spreadsheet
```bash
cargo run --bin parse-traces -- <target> | grep -v toplevel | ./pretty-selectors | clipcopy
```