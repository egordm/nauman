use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use clap::{ArgEnum};
use serde::{Serialize, Deserialize};
use anyhow::{Result, Error};


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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Env {
    #[serde(flatten)]
    base: HashMap<String, String>,
}

impl Env {
    pub fn from_system() -> Self {
        Self { base: std::env::vars().collect() }
    }

    pub fn from_path(path: &PathBuf) -> Result<(Self, Vec<Error>)> {
        let mut base = HashMap::new();
        let mut errors = Vec::new();

        #[allow(deprecated)]
        for item in dotenv::from_path_iter(path)? {
            match item {
                Ok((k, v)) => {
                    base.insert(k, v);
                },
                Err(e) => {
                    errors.push(e.into());
                }
            }
        }

        Ok((Self { base }, errors))
    }

    pub fn insert(&mut self, k: String, v: String) -> Option<String> {
        self.base.insert(k, v)
    }

    #[allow(dead_code)]
    pub fn get(&self, k: &str) -> Option<&String> {
        self.base.get(k)
    }
}

impl From<HashMap<String, String>> for Env {
    fn from(base: HashMap<String, String>) -> Self {
        Self { base }
    }
}

impl Extend<(String, String)> for Env {
    fn extend<T: IntoIterator<Item=(String, String)>>(&mut self, iter: T) {
        self.base.extend(iter)
    }
}

impl FromIterator<(String, String)> for Env {
    fn from_iter<T: IntoIterator<Item=(String, String)>>(iter: T) -> Self {
        Self { base: iter.into_iter().collect() }
    }
}

impl IntoIterator for Env {
    type Item = (String, String);
    type IntoIter = std::collections::hash_map::IntoIter<String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.base.into_iter()
    }
}
