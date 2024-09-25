use anyhow::{Context, Result};
use regex::Regex;
use reqwest;
use serde_json::{json, Value};
use std::env;

fn dfs_has_call(call: &Value, target_address: &str) -> bool {
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
        anyhow::bail!("Usage: {} <block_number> <target_address>", args[0]);
    }
    let block_number: u64 = args[1].parse().context("Failed to parse block number")?;
    let target_address = &args[2];

    let eth_rpc_url = "http://192.168.1.40:8545";

    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "debug_traceBlockByNumber",
        "params": [format!("0x{:x}", block_number), {"tracer": "callTracer"}],
        "id": 1
    });

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(eth_rpc_url)
        .json(&request_body)
        .send()
        .context("Failed to send request to Ethereum node")?;

    let response_text = response.text().context("Failed to get response text")?;

    // Perform a quick check for the target address
    let address_regex =
        Regex::new(&regex::escape(target_address)).context("Failed to create regex")?;

    if !address_regex.is_match(&response_text) {
        // Target address not found, exit early
        return Ok(());
    }

    // If we've reached here, the address is in the response, so we parse the JSON
    let response_body: Value =
        serde_json::from_str(&response_text).context("Failed to parse JSON response")?;

    let traces = response_body
        .get("result")
        .expect("no result in response")
        .as_array()
        .expect("result not an array");

    for trace in traces.iter() {
        if dfs_has_call(&trace["result"], target_address) {
            println!("{}", trace);
        }
    }

    Ok(())
}
