use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    path::PathBuf,
};
use heck::SnakeCase;
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use regex::Regex;
use crate::{
    common::Env,
    LogLevel,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Options {
    /// The default shell which is used to run commands.
    #[serde(default)]
    pub shell: ShellType,
    /// The path to specified shell which is used to run commands.
    pub shell_path: Option<String>,
    /// In dry-run mode, the commands are not executed.
    #[serde(default = "false_default")]
    pub dry_run: bool,
    /// Whether to include ansi escape sequences in the output.
    #[serde(default = "true_default")]
    pub ansi: bool,
    /// Log level used for the output.
    #[serde(default)]
    pub log_level: LogLevel,
    /// Directory where the logs should be written to.
    pub log_dir: Option<String>,
    /// Whether to use system environment variables.
    #[serde(default = "true_default")]
    pub system_env: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            shell: ShellType::default(),
            shell_path: None,
            dry_run: false_default(),
            ansi: true_default(),
            log_level: LogLevel::default(),
            log_dir: None,
            system_env: true_default(),
        }
    }
}

/// Shell to run command with
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ShellType {
    Bash,
    Python,
    Sh,
    Cmd,
    PowerShell,
    Other,
}

lazy_static! {
    static ref COMMAND_PATTERN: Regex = Regex::new(r"^(?<program>((\\[ ])|[A-z0-9/.])+)( +(?<args>.*))?( +|$)").unwrap();
}

impl ShellType {
    /// Returns the program name/path of the shell
    pub fn executable(&self, path: Option<&String>) -> Result<String> {
        if let Some(path) = path {
            let program = COMMAND_PATTERN.captures(&path).and_then(|cap| {
                cap.name("program").map(|login| login.as_str().to_string())
            });

            return program.with_context(|| {
                anyhow!("Invalid command: {}", path)
            });
        } else {
            Ok(match self {
                ShellType::Bash => "bash".to_string(),
                ShellType::Python => "python".to_string(),
                ShellType::Sh => "sh".to_string(),
                ShellType::Cmd => "cmd.exe".to_string(),
                ShellType::PowerShell => "powershell.exe".to_string(),
                ShellType::Other => {
                    return Err(anyhow!("Shell type not supported"));
                }
            })
        }
    }

    pub fn args(&self, _path: Option<&String>, program: String) -> Result<Vec<String>> {
       Ok( match self {
           ShellType::Bash => ["-c".to_string(), program].to_vec(),
           ShellType::Python => ["-c".to_string(), program].to_vec(),
           ShellType::Sh => ["-c".to_string(), program].to_vec(),
           ShellType::Cmd => {
               return Err(anyhow!("Sorry! Windows command shell is not supported yet"));
           },
           ShellType::PowerShell => {
               return Err(anyhow!("Sorry! Windows command shell is not supported yet"));
           },
           ShellType::Other => {
               return Err(anyhow!("Shell type not supported"));
           }
       })
    }
}

impl Default for ShellType {
    fn default() -> Self {
        if cfg!(target_os = "windows") {
            ShellType::Cmd
        } else {
            ShellType::Sh
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shell {
    /// The shell type.
    pub shell: Option<ShellType>,
    /// The shell which is used to run commands.
    pub shell_path: Option<String>,
    /// Shell program that is passed to the shell.
    pub run: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskHandler {
    /// The task handler is a shell command.
    Shell(Shell),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    /// The identifier of the task.
    pub id: Option<String>,
    /// The name of the task.
    pub name: String,
    /// Handler for the task.
    #[serde(flatten)]
    pub handler: TaskHandler,
    /// Environment variable overrides for the task.
    pub env: Option<Env>,
    /// Working directory for the task.
    pub cwd: Option<String>,
    /// Hooks for the task.
    pub hooks: Option<Hooks>,
    /// Execution policy for the task.
    pub policy: Option<ExecutionPolicy>,
}

/// List of tasks
pub type Tasks = Vec<Task>;
/// List of hooks
pub type Hooks = HashMap<Hook, Tasks>;

fn true_default() -> bool {
    true
}

fn false_default() -> bool {
    false
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogOptions {
    /// Whether stdout should be logged.
    #[serde(default = "true_default")]
    pub stdout: bool,
    /// Whether stderr should be logged.
    #[serde(default = "true_default")]
    pub stderr: bool,
    /// Whether hook output should be logged.
    #[serde(default = "true_default")]
    pub hooks: bool,
    /// Whether internal logging should be logged.
    #[serde(default = "true_default")]
    pub internal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHandler {
    /// The file or directory (in split mode) to write to.
    pub output: Option<String>,
    /// Whether logs should be split into multiple files.
    #[serde(default = "false_default")]
    pub split: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum LogHandlerType {
    /// Log to file handler.
    File(FileHandler),
    /// Log to console handler.
    Console,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogHandler {
    /// The type of the log handler.
    #[serde(flatten)]
    pub handler: LogHandlerType,
    /// Log options.
    #[serde(flatten)]
    pub options: LogOptions,
}

/// List of log handlers
pub type LogHandlers = Vec<LogHandler>;

/// Hooks
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Hook {
    /// Before job execution.
    BeforeJob,
    /// After job execution.
    AfterJob,
    /// Before task execution.
    BeforeTask,
    /// After task execution.
    AfterTask,
    /// On task failure (non -zero exit code).
    OnFailure,
    /// On task success (zero exit code).
    OnSuccess,
}

impl Display for Hook {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_snake_case())
    }
}

/// Execution policy
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionPolicy {
    /// Execute the task only if no other task has failed.
    NoPriorFailed,
    /// Execute the task only if prior task has succeeded.
    PriorSuccess,
    /// Execute the task regardless of prior task status.
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
    /// The identifier of the job.
    pub id: Option<String>,
    /// The name of the job.
    pub name: String,
    /// Environment variable overrides for the job.
    pub env: Option<Env>,
    /// Working directory for the job.
    pub cwd: Option<String>,
    /// List of tasks for the job.
    pub tasks: Tasks,
    /// List of global hooks.
    pub hooks: HashMap<Hook, Tasks>,
    /// List of log handlers for the job.
    pub logging: LogHandlers,
    /// Global execution policy for the job.
    #[serde(default)]
    pub policy: ExecutionPolicy,
    /// Global option overrides for the job.
    pub options: Option<Options>,
}

