use std::{
    fs,
};
use crate::{
    execution::{Executor},
    logging::Logger
};
use clap::{AppSettings, Parser};
use crate::common::LogLevel;
use anyhow::{anyhow, Context as AnyhowContext, Result};
use crate::logging::pprint;


mod common;
mod config;
mod logging;
mod flow;
mod execution;

#[derive(Parser)]
#[clap(version = "1.0", author = "Egor D. <egordmitriev2@gmail.com>")]
struct Opts {
    /// Path to job yaml file
    job: String,
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, arg_enum)]
    level: Option<LogLevel>,
    /// Dry run to check job configuration
    dry_run: Option<bool>,
    /// Include ansi colors in output
    ansi: Option<bool>,
    /// Directory to store logs
    log_dir: Option<String>,
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

    let contents = fs::read_to_string(&opts.job)
        .with_context(|| format!("Failed to read file {}", &opts.job))?;

    let job: config::Job = serde_yaml::from_str(&contents)
        .map_err(|e| anyhow!("Failed to parse job file: Error {}", e))?;

    // Merge options
    let mut options = job.options.clone().unwrap_or_default();
    if let Some(level) = opts.level {
        options.log_level = level;
    }
    if let Some(dry_run) = opts.dry_run {
        options.dry_run = dry_run;
    }
    if let Some(ansi) = opts.ansi {
        options.ansi = ansi;
    }
    if let Some(log_dir) = opts.log_dir {
        options.log_dir = Some(log_dir);
    }

    colored::control::set_override(options.ansi);
    let mut logger = Logger::new(job.logging.clone(), options.log_level);
    let flow = flow::Flow::parse(&job)
        .map_err(|e| anyhow!("Failed parsing job: {}", e))?;

    let mut executor = Executor::new(options, &flow)
        .map_err(|e| anyhow!("Failed to create executor: {}", e))?;

    executor.execute(&mut logger)
        .map_err(|e| anyhow!("Fatal error occurred during job execution: {}", e))?;

    Ok(())
}