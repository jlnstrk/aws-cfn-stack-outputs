use std::collections::HashSet;
use std::fs::OpenOptions;
use aws_sdk_cloudformation::Client;
use aws_sdk_cloudformation::operation::describe_stacks::DescribeStacksOutput;
use convert_case::{Case, Casing};
use std::io::Write;

#[tokio::main]
async fn main() {
    let stack = std::env::var("INPUT_STACK").unwrap();
    let prefix = std::env::var("INPUT_PREFIX").unwrap_or(stack.clone());
    let only = std::env::var("INPUT_ONLY").unwrap_or(String::default());

    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);

    let result = client.describe_stacks()
        .stack_name(stack)
        .send().await;

    match result {
        Ok(output) => {
            let kv_pairs = extract_outputs(output);
            let kv_pairs = filter_keys(kv_pairs, only);
            let kv_pairs = transform_keys(prefix, kv_pairs);
            write_to_env(kv_pairs);
        }
        Err(err) => {
            panic!("Failed to get stack description from AWS CloudFormation: {:?}", err.raw_response());
        }
    }
}

fn extract_outputs(output: DescribeStacksOutput) -> Vec<(String, String)> {
    let stacks = output.stacks.unwrap();
    let stack = stacks.first()
        .clone()
        .unwrap();
    let outputs = stack.outputs.clone().unwrap();

    return outputs.into_iter()
        .map(|output| (output.output_key.unwrap(), output.output_value.unwrap()))
        .collect();
}

fn filter_keys(kv_pairs: Vec<(String, String)>, only: String) -> Vec<(String, String)> {
    if only.trim().is_empty() {
        return kv_pairs;
    }

    let only_keys: HashSet<String> = only.split(",")
        .map(|s| s.trim().to_owned())
        .collect();

    kv_pairs.into_iter()
        .filter(|(k, _)| only_keys.contains(k))
        .collect()
}

fn transform_keys(mut prefix: String, kv_pairs: Vec<(String, String)>) -> Vec<(String, String)> {
    if !prefix.is_empty() {
        prefix.push_str("_");
    }
    kv_pairs.into_iter()
        .map(|(k, v)| (format!("{}{}", &prefix, k).to_case(Case::UpperSnake), v))
        .collect()
}

fn write_to_env(vars: Vec<(String, String)>) {
    let github_env_path = std::env::var("GITHUB_ENV").unwrap();
    let mut github_env = OpenOptions::new()
        .write(true)
        .append(true)
        .open(github_env_path)
        .unwrap();

    for (key, value) in vars {
        if let Err(err) = writeln!(github_env, "{}='{}'", key, value) {
            eprintln!("Failed to write to GITHUB_ENV: {}", err);
        }
    }
}
