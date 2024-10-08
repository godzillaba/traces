use std::{collections::HashMap, env, fs::File, io::Write};

use reqwest;

use anyhow::{Context, Result};
use serde_json::{json, Value};

const TARGETS_FILE: &str = "./targets.csv";

const SELECTOR_CACHE: &str = "./data/selector_cache.tsv";
const CONTRACT_NAME_CACHE: &str = "./data/contract_name_cache.tsv";

#[derive(Debug, Eq, PartialEq, Hash)]
struct UglyCall {
    call_type: String,
    selector: String,
    from: String,
    to: String,
}

impl UglyCall {
    fn from_json(value: &Value) -> Self {
        Self {
            call_type: value["type"].as_str().unwrap().to_string(),
            selector: if value["input"].as_str().unwrap_or_default().len() >= 10 {
                value["input"].as_str().unwrap_or_default()[0..10].to_string()
            } else {
                "".to_string()
            },
            from: value["from"].as_str().unwrap_or_default().to_string(),
            to: value["to"].as_str().unwrap_or_default().to_string(),
        }
    }
}

struct PrettyCall {
    call_type: String,
    signature: String,
    from: String,
    to: String,
    from_name: String,
    to_name: String,
}

impl PrettyCall {
    fn from_ugly_call(
        ugly_call: UglyCall,
        contract_name_cache: &mut HashMap<String, String>,
        selector_cache: &mut HashMap<String, String>,
    ) -> Result<Self> {
        let from_name = fetch_contract_name(&ugly_call.from, contract_name_cache)?;
        let to_name = fetch_contract_name(&ugly_call.to, contract_name_cache)?;
        let signature = fetch_signature(&ugly_call.selector, selector_cache)?;

        Ok(Self {
            call_type: ugly_call.call_type,
            signature,
            from: ugly_call.from,
            to: ugly_call.to,
            from_name,
            to_name,
        })
    }
}

fn load_targets() -> Result<Vec<String>> {
    let targets = std::fs::read_to_string(TARGETS_FILE)
        .context("Failed to read targets file")?
        .to_lowercase()
        .lines()
        .map(|line| {
            line.split(',')
                .collect::<Vec<&str>>()
                .get(0)
                .unwrap()
                .to_string()
        })
        .collect::<Vec<String>>();

    Ok(targets)
}

fn load_selector_cache() -> Result<HashMap<String, String>> {
    match std::fs::read_to_string(SELECTOR_CACHE) {
        Err(_) => {
            File::create(SELECTOR_CACHE).context("Failed to create selector cache file")?;
            Ok(HashMap::new())
        }
        Ok(contents) => Ok(contents
            .lines()
            .map(|line| line.split('\t').collect::<Vec<&str>>())
            .map(|line| {
                (
                    line.get(0).unwrap().to_string(),
                    line.get(1).unwrap().to_string(),
                )
            })
            .collect::<HashMap<String, String>>()),
    }
}

fn load_contract_name_cache() -> Result<std::collections::HashMap<String, String>> {
    match std::fs::read_to_string(CONTRACT_NAME_CACHE) {
        Err(_) => {
            File::create(CONTRACT_NAME_CACHE)
                .context("Failed to create contract name cache file")?;
            Ok(std::collections::HashMap::new())
        }
        Ok(contents) => Ok(contents
            .lines()
            .map(|line| line.split('\t').collect::<Vec<&str>>())
            .map(|line| {
                (
                    line.get(0).unwrap().to_string(),
                    line.get(1).unwrap().to_string(),
                )
            })
            .collect::<std::collections::HashMap<String, String>>()),
    }
}

fn put_to_contract_name_cache(address: &str, contract_name: &str) -> Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(CONTRACT_NAME_CACHE)
        .context("Failed to open contract name cache file")?;

    file.write_fmt(format_args!("{}\t{}\n", address, contract_name))
        .context("Failed to write to contract name cache file")?;

    Ok(())
}

fn put_to_selector_cache(selector: &str, signature: &str) -> Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(SELECTOR_CACHE)
        .context("Failed to open selector cache file")?;

    file.write_fmt(format_args!("{}\t{}\n", selector, signature))
        .context("Failed to write to selector cache file")?;

    Ok(())
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

// fetch the signature from selector
fn fetch_signature(selector: &str, selector_cache: &mut HashMap<String, String>) -> Result<String> {
    // curl -X 'GET' \
    //     'https://api.openchain.xyz/signature-database/v1/lookup?function=0x4870496f&filter=true' \
    //     -H 'accept: application/json'

    // if the selector is empty, return empty string
    if selector.is_empty() {
        return Ok("".to_string());
    }

    // if the selector is in the cache, return the signature
    if let Some(signature) = selector_cache.get(selector) {
        return Ok(signature.to_string());
    }

    // fetch the signature from the API
    let url = format!(
        "https://api.openchain.xyz/signature-database/v1/lookup?function={}&filter=true",
        selector
    );

    let response = reqwest::blocking::get(&url).context("Failed to fetch signature")?;

    let response_body: Value =
        serde_json::from_str(&response.text().context("Failed to read response body")?)
            .context("Failed to parse JSON response")?;

    let signature = response_body["result"].as_object().unwrap()["function"]
        .as_object()
        .unwrap()[selector]
        .as_array()
        .unwrap()[0]
        .as_object()
        .unwrap()["name"]
        .as_str()
        .unwrap()
        .to_string();

    // put the signature into the cache
    selector_cache.insert(selector.to_string(), signature.to_string());
    put_to_selector_cache(selector, &signature)?;

    Ok(signature)
}

// fetch the contract name
fn fetch_contract_name(
    address: &str,
    contract_name_cache: &mut HashMap<String, String>,
) -> Result<String> {
    if address.is_empty() {
        return Ok("".to_string());
    }

    if let Some(contract_name) = contract_name_cache.get(address) {
        return Ok(contract_name.to_string());
    }

    let url = format!(
        "https://api.etherscan.io/api?module=contract&action=getsourcecode&address={}&apikey={}",
        address,
        env::var("ETHERSCAN_API_KEY").context("Missing ETHERSCAN_API_KEY")?
    );
    let response = reqwest::blocking::get(&url).context("Failed to fetch contract name")?;
    let response_body: Value =
        serde_json::from_str(&response.text().context("Failed to read response body")?)
            .context("Failed to parse JSON response")?;

    let result = response_body["result"]
        .as_array()
        .context("No result in response body")?
        .get(0)
        .context("No result in response body")?;

    let mut contract_name = result
        .get("ContractName")
        .context("No ContractName in response body")?
        .as_str()
        .context("ContractName is not a string")?
        .to_string();

    let implementation = result
        .get("Implementation")
        .context("No Implementation in response body")?
        .as_str()
        .context("Implementation is not a string")?;

    if !implementation.is_empty() {
        let impl_name = fetch_contract_name(implementation, contract_name_cache)?;
        contract_name = contract_name.to_string() + " -> " + &impl_name;
    }

    // if the contract is a proxy, fetch the implementation contract name

    put_to_contract_name_cache(address, contract_name.as_str())?;

    contract_name_cache.insert(address.to_string(), contract_name.to_string());

    Ok(contract_name.to_string())
}

fn main() -> Result<()> {
    // load targets into a vec
    let targets = load_targets()?;

    // load selector cache into a hashmap
    let mut selector_cache = load_selector_cache()?;

    // load contract name cache into a hashmap
    let mut contract_name_cache = load_contract_name_cache()?;

    // read stdin line by line
    let mut input = String::new();
    while let Ok(n) = std::io::stdin().read_line(&mut input) {
        if n == 0 {
            break;
        }

        // if input is "DONE", break
        if input.trim() == "DONE" {
            break;
        }

        // parse the line as JSON
        let mut response_body: Value =
            serde_json::from_str(&input).context("Failed to parse JSON input")?;

        let trace = response_body
            .get_mut("result")
            .context("no result in response")?;

        // add a toplevel type to the trace
        *trace.get_mut("type").unwrap() = json!("toplevel");

        let mut flattened_calls = Vec::new();
        dfs_flatten_calls(trace, &mut flattened_calls);

        for call in flattened_calls.iter() {
            let ugly_call = UglyCall::from_json(call);

            if targets.contains(&ugly_call.to) || targets.contains(&ugly_call.from) {
                let pretty_call = PrettyCall::from_ugly_call(
                    ugly_call,
                    &mut contract_name_cache,
                    &mut selector_cache,
                )?;
                println!(
                    "{}\t{}\t{}\t{}\t{}\t{}",
                    pretty_call.call_type,
                    pretty_call.signature,
                    pretty_call.from,
                    pretty_call.to,
                    pretty_call.from_name,
                    pretty_call.to_name,
                );
            }
        }

        input.clear();
    }

    Ok(())
}
