use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::common::Env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: Option<String>,
    pub name: String,
    pub run: String,
    pub env: Option<Env>,
    pub cwd: Option<String>,
}

pub type Tasks = Vec<Task>;

fn true_default() -> bool {
    true
}

fn false_default() -> bool {
    false
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogOptions {
    #[serde(default = "true_default")]
    pub stdout: bool,
    #[serde(default = "true_default")]
    pub stderr: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHandler {
    pub output: Option<String>,
    #[serde(default = "false_default")]
    pub split: bool,
}

#[serde(rename_all = "snake_case", tag = "type")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogHandlerType {
    File(FileHandler),
    Console,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogHandler {
    #[serde(flatten)]
    pub handler: LogHandlerType,
    #[serde(flatten)]
    pub options: LogOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "true_default")]
    pub ansi: bool,
    pub handlers: Vec<LogHandler>,
}

#[serde(rename_all = "snake_case")]
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Hook {
    Before,
    After,
    BeforeTask,
    AfterTask
}

impl ToString for Hook {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
    pub name: String,
    pub env: Option<Env>,
    pub cwd: Option<String>,
    pub tasks: Tasks,
    pub hooks: HashMap<Hook, Tasks>,
    pub logging: LoggingConfig,
}

