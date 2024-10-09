# Traces

Use this tool to find and analyze traces that touch certain target accounts.

## Usage

This assumes your rpc is `localhost:8545`. To change, edit `./cmd/find-traces`

### 1. Set Targets

Set a list of `target_address,target_label` in `targets.csv`

### 2. Gather Traces

To gather all the traces that touch the targets, use

```bash
./cmd/find-traces <start block> <end block>
```

Results will land in `data/find-traces-results.txt`, with each transaction trace object on its own line.

You can adjust the number of parallel jobs by changing `-j 16` in `./find-traces`.

This command can be safely stopped and will pick up where it left off when restarted. To completely restart, first remove `data/find-traces-joblog.txt`

### 3. Parse Gathered Traces

While `./cmd/find-traces` is still running or has finished, run:

```bash
./cmd/prettify-found-traces
```

This will watch for new traces and find the function signatures and contract names. Its output is `data/pretty-traces.tsv`

### 4. Create a Spreadsheet

Once 2 and 3 are done, use `./cmd/create-spreadsheet` to create `data/spreadsheet.xlsx`