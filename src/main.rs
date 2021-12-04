use std::{
    fs,
};
use crate::execution::execute_flow;

mod common;
mod config;
mod output;
mod flow;
mod execution;
mod pprint;
mod logging;
mod dual_output;

fn main() {
    let contents = fs::read_to_string("example/test.yml")
        .expect("Something went wrong reading the file");

    let job: config::Job = serde_yaml::from_str(&contents).unwrap();
    let flow = flow::Flow::from_job(&job).unwrap();

    execute_flow(&flow).unwrap();
}
