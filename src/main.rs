use std::{
    fs,
};
use crate::execution::execute_flow;
use crate::logging::Logger;

mod common;
mod config;
mod logging;
mod flow;
mod execution;
mod pprint;

fn main() {
    let contents = fs::read_to_string("example/test.yml")
        .expect("Something went wrong reading the file");

    let job: config::Job = serde_yaml::from_str(&contents).unwrap();
    let mut logger = Logger::new(job.logging.clone());
    let flow = flow::Flow::parse(&job).unwrap();

    execute_flow(&flow, &mut logger).unwrap();
}
