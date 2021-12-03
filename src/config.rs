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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLogger {
    suffix: Option<String>,
    split: bool,
}

#[serde(rename_all = "snake_case", tag = "type")]
#[derive(Debug, Serialize, Deserialize)]
pub enum Loggers {
    File(FileLogger)
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
    pub logging: Vec<Loggers>,
}

