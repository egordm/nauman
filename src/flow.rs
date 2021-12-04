use std::collections::HashMap;
use crate::config;
use anyhow::{anyhow, Result};
use crate::common::Env;

pub type CommandId = String;
pub type Dependencies = HashMap<CommandId, Command>;

#[derive(Debug)]
pub struct Command {
    pub name: String,
    pub run: String,
    pub env: Env,
    pub cwd: Option<String>,
}

impl From<&config::Task> for Command {
    fn from(t: &config::Task) -> Self {
        Self {
            name: t.name.clone(),
            run: t.run.clone(),
            env: t.env.clone().unwrap_or(Env::default()),
            cwd: t.cwd.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Routine {
    commands: Vec<CommandId>,
}

impl Default for Routine {
    fn default() -> Self {
        Self::new()
    }
}

fn generate_task_id(task: &config::Task, counter: usize, prefix: impl ToString) -> String {
    format!("{}-{:03}-{}", prefix.to_string(), counter, task.name)
}

impl Routine {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn new_singleton(command_id: CommandId) -> Self {
        Self {
            commands: vec![command_id],
        }
    }

    pub fn from_task_list(
        tasks: &[config::Task],
        dependencies: &mut Dependencies,
        prefix: impl ToString,
    ) -> Result<Self> {
        let mut counter = 0;
        let mut commands = Vec::new();

        for task in tasks {
            let task_id = task.id.clone()
                .unwrap_or_else(|| generate_task_id(task, counter, prefix.to_string()));

            // TODO: handle this in a more verbose way
            if let Some(_old_commmand) = dependencies.insert(task_id.clone(), Command::from(task)) {
                return Err(anyhow!("Task {} has a duplicate id", task.name));
            }

            commands.push(task_id);
            counter += 1;
        }

        Ok(Self { commands })
    }

    pub fn chain(self, other: Self) -> Self {
        Self {
            commands: self.commands.into_iter()
                .chain(other.commands.into_iter()).collect(),
        }
    }

    pub fn extend(&mut self, other: Self) {
        self.commands.extend(other.commands.into_iter());
    }
}

#[derive(Debug)]
pub struct Flow {
    pub dependencies: Dependencies,
    pub flow: Routine,
    pub env: Env,
    pub cwd: Option<String>,
}

const MAIN_ROUTINE_NAME: &str = "main";

impl Flow {
    pub fn from_job(
        job: &config::Job
    ) -> Result<Self> {
        let mut dependencies = HashMap::new();
        let mut flow = Routine::new();
        let mut hooks: HashMap<config::Hook, Routine> = HashMap::new();

        // Build hooks
        for (hook, tasks) in job.hooks.iter() {
            let routine = Routine::from_task_list(tasks, &mut dependencies, hook.to_string())?;
            hooks.insert(hook.clone(), routine);
        }

        // Schedule before hook
        if let Some(before_hook) = hooks.get(&config::Hook::Before) {
            flow.extend(before_hook.clone());
        }

        // Build and schedule main tasks
        let main_routine = Routine::from_task_list(&job.tasks, &mut dependencies, MAIN_ROUTINE_NAME)?;
        let before_task = hooks.get(&config::Hook::BeforeTask).cloned().unwrap_or_default();
        let after_task = hooks.get(&config::Hook::AfterTask).cloned().unwrap_or_default();
        for command_id in main_routine.commands.into_iter() {
            flow.extend(
                before_task.clone()
                    .chain(Routine::new_singleton(command_id))
                    .chain(after_task.clone())
            );
        }

        // Schedule after hook
        if let Some(before_hook) = hooks.get(&config::Hook::After) {
            flow.extend(before_hook.clone());
        }

        Ok(Self {
            dependencies,
            flow,
            env: job.env.clone().unwrap_or_default(),
            cwd: job.cwd.clone(),
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = (&CommandId, &Command)> {
        self.flow.commands.iter().map(move |command_id| {
            (command_id, self.dependencies.get(command_id).unwrap())
        })
    }
}