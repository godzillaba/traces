use anyhow::{Context, Result};
use serde_json::Value;
use std::env;
use std::io::{self, Read};

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
    let target_address = env::args().nth(1).context("Missing target address argument")?;

    let mut input = String::new();
    io::stdin().read_to_string(&mut input).context("Failed to read from stdin")?;
    input.make_ascii_lowercase();

    // Quick check for the target address (case-insensitive), target already lower case
    if !input.contains(&target_address) {
        return Ok(());
    }

    // If we've reached here, the address is in the input, so we parse the JSON
    let response_body: Value =
        serde_json::from_str(&input).context("Failed to parse JSON input")?;

    let traces = response_body
        .get("result")
        .expect("no result in response")
        .as_array()
        .expect("result not an array");

    for trace in traces.iter() {
        if dfs_has_call(&trace["result"], &target_address) {
            println!("{}", trace);
        }
    }

    Ok(())
}
