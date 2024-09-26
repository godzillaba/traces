use std::env;

use anyhow::{Context, Result};
use serde_json::{json, Value};

const FILE: &str = "./data/find-traces-results.txt";

#[derive(Debug, Eq, PartialEq, Hash)]
struct Call {
    call_type: String,
    selector: Option<String>,
    from: String,
}

impl Call {
    fn from_json(value: &Value) -> Self {
        Self {
            call_type: value["type"].as_str().unwrap_or_default().to_string(),
            selector: if value["input"].as_str().unwrap_or_default().len() >= 10 {
                Some(value["input"].as_str().unwrap_or_default()[0..10].to_string())
            } else {
                None
            },
            from: value["from"].as_str().unwrap_or_default().to_string(),
        }
    }
}

// run a dfs to flatten the calls
fn dfs_flatten_calls<'a>(call: &'a Value, call_set: &mut Vec<&'a Value>) {
    call_set.push(call);

    if let Some(subcalls) = call["calls"].as_array() {
        for subcall in subcalls.iter() {
            dfs_flatten_calls(subcall, call_set);
        }
    }
}

fn main() -> Result<()> {
    let target_address = env::args().nth(1).context("Missing target address argument")?.to_ascii_lowercase();
    let input = std::fs::read_to_string(FILE).context("Failed to read file")?;
    let mut call_set = std::collections::HashSet::new();

    for line in input.lines() {
        let mut response_body: Value = serde_json::from_str(line).context("Failed to parse JSON input")?;

        let trace = response_body
            .get_mut("result")
            .context("no result in response")?;
        
        // add a toplevel type to the trace
        *trace.get_mut("type").unwrap() = json!("toplevel");

        let mut flattened_calls = Vec::new();
        dfs_flatten_calls(trace, &mut flattened_calls);

        for call in flattened_calls.iter() {
            if let Some(to) = call["to"].as_str() {
                if to.to_ascii_lowercase() == target_address {
                    call_set.insert(Call::from_json(call));
                }
            }
        }
    }

    // print the set as csv
    for call in call_set.iter() {
        println!("{},{},{}", call.call_type, call.selector.as_deref().unwrap_or_default(), call.from);
    }

    Ok(())
}
