use reqwest;
use serde_json::{json, Value};
use std::env;
use anyhow::Result;

fn dfs_has_call(call: &Value, target_address: &str) -> bool {
    // call has .calls and .to fields
    if let Some(to) = call["to"].as_str() {
        if to == target_address {
            return true;
        }
    }

    if let Some(calls) = call["calls"].as_array() {
        for subcall in calls.iter() {
            if dfs_has_call(subcall, target_address) {
                return true;
            }
        }
    }

    false
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <block_number> <target_address>", args[0]);
        std::process::exit(1);
    }
    let block_number: u64 = args[1].parse()?;
    let target_address = &args[2];

    let eth_rpc_url = "http://192.168.1.40:8545";

    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "debug_traceBlockByNumber",
        "params": [format!("0x{:x}", block_number), {"tracer": "callTracer"}],
        "id": 1
    });

    let client = reqwest::blocking::Client::new();
    let response = client.post(eth_rpc_url)
        .json(&request_body)
        .send()?;

    let response_body: Value = response.json()?;

    if let Some(result) = response_body.get("result") {
        if let Some(traces) = result.as_array() {
            for trace in traces.iter() {
                if dfs_has_call(&trace["result"], target_address) {
                    println!("{}", trace);
                }
            }
        }
    }

    Ok(())
}