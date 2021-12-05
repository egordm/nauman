use std::collections::HashMap;
use std::str::FromStr;
use clap::{ArgEnum};
use serde::{Serialize, Deserialize};

pub type Env = HashMap<String, String>;


#[derive(ArgEnum, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LogLevel {
    #[clap(name = "debug")]
    Debug = 4,
    #[clap(name = "info")]
    Info = 3,
    #[clap(name = "warn")]
    Warn = 2,
    #[clap(name = "error")]
    Error = 1,
}

impl FromStr for LogLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err(()),
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}