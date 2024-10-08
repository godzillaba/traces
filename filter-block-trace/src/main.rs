use anyhow::{Context, Result};
use serde_json::Value;
use std::env;
use std::io::{self, Read};

fn dfs_has_call(call: &Value, target_addresses: &Vec<String>) -> bool {
    if let Some(to) = call["to"].as_str() {
        if target_addresses.contains(&to.to_string()) {
            return true;
        }
    }

    if let Some(calls) = call["calls"].as_array() {
        for subcall in calls.iter() {
            if dfs_has_call(subcall, target_addresses) {
                return true;
            }
        }
    }

    false
}

fn main() -> Result<()> {
    let target_addresses = env::args().skip(1).collect::<Vec<String>>();

    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .context("Failed to read from stdin")?;
    input.make_ascii_lowercase();

    // Quick check for the target addresses (case-insensitive), targets already lower case
    let found = target_addresses
        .iter()
        .any(|target_address| input.contains(target_address));
    if !found {
        return Ok(());
    }

    // If we've reached here, one of the addresses is in the input, so we parse the JSON
    let response_body: Value =
        serde_json::from_str(&input).context("Failed to parse JSON input")?;

    let traces = response_body
        .get("result")
        .expect("no result in response")
        .as_array()
        .expect("result not an array");

    for trace in traces.iter() {
        if dfs_has_call(&trace["result"], &target_addresses) {
            println!("{}", trace);
        }
    }

    Ok(())
}
