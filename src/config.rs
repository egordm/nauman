use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use heck::SnakeCase;
use serde::{Serialize, Deserialize};
use crate::common::Env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Options {
    shell: Option<String>,
    #[serde(default = "false_default")]
    dry_run: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            shell: None,
            dry_run: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shell {
    pub shell: Option<String>,
    pub run: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskHandler {
    Shell(Shell),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: Option<String>,
    pub name: String,
    #[serde(flatten)]
    pub handler: TaskHandler,
    pub env: Option<Env>,
    pub cwd: Option<String>,
    pub hooks: Option<Hooks>,
    pub policy: Option<ExecutionPolicy>,
}

pub type Tasks = Vec<Task>;
pub type Hooks = HashMap<Hook, Tasks>;

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
    BeforeJob,
    AfterJob,
    BeforeTask,
    AfterTask,
    OnFailure,
    OnSuccess,
}

impl Display for Hook {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_snake_case())
    }
}

#[serde(rename_all = "snake_case")]
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExecutionPolicy {
    NoPriorFailed,
    PriorSuccess,
    Always,
}

impl Default for ExecutionPolicy {
    fn default() -> Self {
        ExecutionPolicy::NoPriorFailed
    }
}

impl Display for ExecutionPolicy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_snake_case())
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
    #[serde(default)]
    pub policy: ExecutionPolicy,
    pub options: Option<Options>,
}

