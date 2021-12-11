#[doc = include_str!("../README.md")]


use std::{
    fs,
};
use std::error::Error;
use std::path::PathBuf;
use crate::{
    execution::{Executor},
    logging::Logger
};
use clap::{Parser, ValueHint};
use crate::common::LogLevel;
use anyhow::{anyhow, Context as AnyhowContext, Result};
use crate::config::LogHandler;
use crate::logging::pprint;

mod common;
mod config;
mod logging;
mod flow;
mod execution;
mod utils;

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
    where
        T: std::str::FromStr,
        T::Err: Error + Send + Sync + 'static,
        U: std::str::FromStr,
        U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

#[derive(Parser)]
#[clap(version = "1.0", author = "Egor D. <egordmitriev2@gmail.com>")]
struct Opts {
    /// Path to job yaml file
    job: String,
    /// A level of verbosity, and can be used multiple times (default: info)
    #[clap(short, long, arg_enum)]
    level: Option<LogLevel>,
    /// Dry run to check job configuration (default: false)
    #[clap(long)]
    dry_run: bool,
    /// Include ansi colors in output (default: true)
    #[clap(long)]
    ansi: Option<bool>,
    /// Directory to store logs in (default: current directory)
    #[clap(long, value_hint = ValueHint::FilePath)]
    log_dir: Option<String>,
    /// Whether to use system environment variables (default: true)
    #[clap(long)]
    system_env: Option<bool>,
    /// List of env variable overrides
    #[clap(short = 'e', parse(try_from_str = parse_key_val), multiple_occurrences(true), number_of_values = 1)]
    env: Vec<(String, String)>,
}

fn main() {
   match run() {
       Ok(_) => {}
       Err(e) => {
           eprintln!("{}", pprint::error(&format!("{}", e)));
           std::process::exit(1);
       }
   }
}

fn run() -> Result<()> {
    let opts: Opts = Opts::parse();

    // Read and parse the job file
    let contents = fs::read_to_string(&opts.job)
        .with_context(|| format!("Failed to read job file: {}", &opts.job))?;
    let mut job: config::Job = serde_yaml::from_str(&contents)
        .map_err(|e| anyhow!("Failed to parse job file: Error {}", e))?;

    // Update job
    let filename = PathBuf::from(&opts.job).file_stem().and_then(|f| f.to_str()).unwrap().to_string();
    job.id = Some(job.id.unwrap_or(filename));

    // Merge options
    // TODO: move this to a separate function
    let mut options = job.options.clone().unwrap_or_default();
    if let Some(level) = opts.level {
        options.log_level = level;
    }
    if opts.dry_run {
        options.dry_run = opts.dry_run;
    }
    if let Some(ansi) = opts.ansi {
        options.ansi = ansi;
    }
    if let Some(log_dir) = opts.log_dir {
        options.log_dir = Some(log_dir);
    }
    if let Some(system_env) = opts.system_env {
        options.system_env = system_env;
    }
    if let Some(env) = job.env.as_mut() {
        env.extend(opts.env);
    } else {
        job.env = Some(opts.env.iter().cloned().collect());
    };

    // Setup the logger
    colored::control::set_override(options.ansi);
    let logging_handlers = if let Some(handlers) = job.logging.clone() {
        handlers
    } else {
        vec![LogHandler::default_console()]
    };
    let mut logger = Logger::new(logging_handlers, options.log_level);

    // Parse the job to a flow
    let flow = flow::Flow::parse(&job)
        .map_err(|e| anyhow!("Failed parsing job: {}", e))?;

    // Create an executor for the given flow
    let mut executor = Executor::new(options, &flow)
        .map_err(|e| anyhow!("Failed to create executor: {}", e))?;

    // Execute the flow
    executor.execute(&mut logger)
        .map_err(|e| anyhow!("Fatal error occurred during job execution: {}", e))?;

    Ok(())
}