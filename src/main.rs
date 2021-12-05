use std::{
    fs,
};
use crate::execution::{ Executor};
use crate::logging::Logger;

mod common;
mod config;
mod logging;
mod flow;
mod execution;

fn main() {
    let contents = fs::read_to_string("example/test.yml")
        .expect("Something went wrong reading the file");

    let job: config::Job = serde_yaml::from_str(&contents).unwrap();
    let options = job.options.clone().unwrap_or_default();
    let mut logger = Logger::new(job.logging.clone());
    let flow = flow::Flow::parse(&job).unwrap();

    Executor::new(
        options,
        &flow
    ).unwrap().execute(&mut logger).unwrap();
}
