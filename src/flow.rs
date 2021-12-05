use std::collections::HashMap;
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use crate::{
    common::Env,
    config,
    config::{Hook}
};
use crate::config::{ExecutionPolicy, TaskHandler};
use crate::execution::ExecutionResult;

pub type CommandId = String;
pub type RoutineId = String;
pub type Dependencies = HashMap<CommandId, Command>;
pub type Routines = HashMap<RoutineId, Routine>;
pub type Hooks = HashMap<Hook, RoutineId>;


const MAIN_ROUTINE_NAME: &str = "main";

lazy_static! {
    static ref IDENTIFIER_REGEX: Regex = Regex::new(r"[^a-zA-Z0-9_\-]").unwrap();
}

/// Formats text as a valid identifier.
/// Which should also be valid as a system file name.
fn format_identifier(text: &str) -> String {
    IDENTIFIER_REGEX.replace_all(
        &text.to_lowercase().replace(" ", "-"), ""
    ).to_string()
}

/// Generates a unique identifier for a command given a name and counter in the current routine
fn generate_id(name: &str, counter: usize, prefix: &str) -> String {
    format!("{}{:03}_{}", prefix, counter, format_identifier(name))
}

/// A command is a task that can be executed by the system.
#[derive(Debug, Clone)]
pub struct Command {
    /// The command's name.
    pub name: String,
    /// The command's handler
    pub handler: TaskHandler,
    /// Environment variable overrides
    pub env: Env,
    /// Working directory to execute the command in.
    pub cwd: Option<String>,
    /// Whether the command is a hook.
    pub is_hook: bool,
    /// The command's hook overrides
    pub hooks: Hooks,
    /// The command's execution policy
    pub policy: ExecutionPolicy,
}

#[derive(Debug, Clone)]
pub struct Routine {
    /// List of commands to execute.
    commands: Vec<CommandId>,
    /// Whether the routine is a hook.
    pub is_hook: bool,
}

/// A flow is a collection of routines and commands representing a job
#[derive(Debug, Clone)]
pub struct Flow {
    /// The flow's name.
    pub id: String,
    /// List of tasks with corresponding command ids
    pub dependencies: Dependencies,
    /// List of routines with corresponding routine ids
    pub routines: Routines,
    /// List of global hooks
    pub hooks: Hooks,
    /// Collection of job specific environment variable overrides
    pub env: Env,
    /// Working directory to execute the job in.
    pub cwd: Option<String>,
}

impl Flow {
    /// Creates a new flow from the job configuration.
    pub fn parse(job: &config::Job) -> Result<Self> {
        FlowBuilder::new().parse_flow(job)
    }

    /// Get command by id
    pub fn command(&self, command_id: &CommandId) -> Option<&Command> {
        self.dependencies.get(command_id)
    }

    /// Iterates through flow's commands including hook logic
    pub fn iter(&self) -> FlowIterator {
        FlowIterator::new(self)
    }
}

#[derive(Debug)]
pub struct FlowBuilder {
    /// List of tasks with corresponding command ids
    pub dependencies: Dependencies,
    /// List of routines with corresponding routine ids
    pub routines: Routines,
    /// Global execution policy
    pub policy: ExecutionPolicy,
}

impl FlowBuilder {
    pub fn new() -> Self {
        FlowBuilder {
            dependencies: HashMap::new(),
            routines: HashMap::new(),
            policy: ExecutionPolicy::default(),
        }
    }

    /// Parses a routine from a list of tasks
    pub fn parse_routine(
        &mut self,
        tasks: &config::Tasks,
        prefix: &str,
        is_hook: bool,
    ) -> Result<Routine> {
        let mut counter = 0;
        let mut commands = Vec::new();

        for task in tasks {
            let task_name = task.get_name();
            let task_id = task.id.clone()
                .unwrap_or_else(|| generate_id(&task_name, counter, prefix));

            // TODO: handle this in a more verbose way
            let command = self.parse_command(task, &task_id, is_hook)?;
            if let Some(_) = self.dependencies.insert(task_id.clone(), command) {
                return Err(anyhow!("Task {} has a duplicate id", task_name));
            }

            commands.push(task_id);
            counter += 1;
        }

        Ok(Routine { commands,  is_hook })
    }

    /// Parses a list of hooks
    pub fn parse_hooks(
        &mut self,
        hooks: &config::Hooks,
        prefix: &str,
        is_hook: bool,
    ) -> Result<Hooks> {
        if is_hook {
            return Err(anyhow!("Hooks cannot be nested!"));
        }

        let mut result = Hooks::new();

        for (hook, tasks) in hooks {
            let routine_id = format!("{}{}", prefix, &hook);
            let routine = self.parse_routine(tasks, &routine_id, true)?;
            self.routines.insert(routine_id.clone(), routine);
            result.insert(hook.clone(), routine_id);
        }

        Ok(result)
    }

    /// Parses a command from a task
    pub fn parse_command(
        &mut self,
        task: &config::Task,
        prefix: &str,
        is_hook: bool,
    ) -> Result<Command> {
        let hooks = if let Some(hooks) = &task.hooks {
            self.parse_hooks(hooks, prefix, is_hook)?
        } else {
            HashMap::new()
        };

        Ok(Command {
            name: task.get_name(),
            handler: task.handler.clone(),
            env: task.env.clone().unwrap_or(Env::default()),
            cwd: task.cwd.clone(),
            is_hook,
            hooks,
            policy: task.policy.unwrap_or(self.policy),
        })
    }

    /// Parses a flow from a job configuration
    pub fn parse_flow(
        mut self,
        job: &config::Job,
    ) -> Result<Flow> {
        // Set the job identifier if not yet set (usually the filename unless it is overridden)
        let id = job.id.clone().unwrap_or_else(|| format_identifier(&job.name));

        // Set the global execution policy
        self.policy = job.policy;

        // Parse the global hooks
        let hooks = if let Some(hooks) = &job.hooks {
            self.parse_hooks(hooks, "", false)?
        } else {
            HashMap::new()
        };

        // Parse the main routine and add it to the list of routines
        let main_routine = self.parse_routine(&job.tasks, "", false)?;
        self.routines.insert(MAIN_ROUTINE_NAME.to_string(), main_routine);

        Ok(Flow {
            id,
            dependencies: self.dependencies,
            routines: self.routines,
            env: job.env.clone().unwrap_or_default(),
            cwd: job.cwd.clone(),
            hooks,
        })
    }
}

/// Stack item encoding information about current execution state of a routine
#[derive(Debug, Clone)]
pub struct StackItem {
    /// Current routine
    pub routine: RoutineId,
    /// Current position pointing to a command in routine
    /// value of -1 means that routine has yet to schedule before hooks
    pub position: i32,
    /// Whether hooks for the current command have been scheduled
    pub scheduled: bool,
    /// Whether the current command is a hook
    pub is_hook: bool,
    /// Length of the current routine
    pub length: i32,
    /// Reference to main current (non hook) routine command
    /// If none, then the main routine is finished
    pub focus_command: Option<CommandId>,
}

/// Iterator over flow's commands including hook logic
pub struct FlowIterator<'a> {
    flow: &'a Flow,
    routine_stack: Vec<StackItem>,
}

impl <'a> FlowIterator<'a> {
    pub fn new(flow: &'a Flow) -> Self {
        let mut res = FlowIterator {
            flow,
            routine_stack: Vec::new(),
        };
        res.push(MAIN_ROUTINE_NAME.to_string(), None);
        res
    }

    /// Returns stack head item
    fn head(&self) -> Option<StackItem> {
        self.routine_stack.last().cloned()
    }

    /// Returns mutable ref to the stack head item
    fn head_mut(&mut self) -> Option<&mut StackItem> {
        self.routine_stack.last_mut()
    }

    /// Increments the current routine position
    fn increment_position(&mut self) {
        if let Some(item) = self.head_mut() {
            item.position += 1;
            item.scheduled = false;
        }
    }

    /// Sets current stack item to scheduled
    fn set_scheduled(&mut self) {
        if let Some(item) = self.head_mut() {
            item.scheduled = true;
        }
    }

    /// Returns a routine given its id
    fn routine(&self, routine_id: &RoutineId) -> &Routine {
        self.flow.routines.get(routine_id).expect("Routine not found")
    }

    /// Returns a command given its id
    fn command(&self, command_id: &CommandId) -> &Command {
        self.flow.dependencies.get(command_id).expect("Command not found")
    }

    /// Returns info about current command given a stack item
    fn get_command(&self, head: &StackItem) -> Option<(&CommandId, &Command)> {
        let routine = self.routine(&head.routine);
        let command_id = routine.commands.get(head.position as usize).expect("Command not found");
        let command = self.command(&command_id);
        Some((command_id, command))
    }

    /// Pushes a new routine onto the stack
    fn push(&mut self, routine_id: RoutineId, focus: Option<CommandId>) {
        let routine = self.routine(&routine_id);
        let item = StackItem{
            routine: routine_id,
            position: -1,
            scheduled: false,
            is_hook: routine.is_hook,
            length: routine.commands.len() as i32,
            focus_command: focus,
        };

        self.routine_stack.push(item);
    }

    /// Pops the current routine from the stack
    fn pop(&mut self) {
        self.routine_stack.pop();
    }

    /// Pushes a command result and subsequent commands to the stack
    pub fn push_result(
        &mut self,
        command_id: &CommandId,
        result: &ExecutionResult
    ) {
        let hook_type = if result.is_success() { Hook::OnSuccess } else { Hook::OnFailure };

        let command = self.command(command_id);
        if !command.is_hook {
            // Add a task-specific hook
            if let Some(hook) = command.hooks.get(&hook_type).cloned() {
                self.push(hook, Some(command_id.clone()));
            }
            // Add a global hook
            if let Some(hook) = self.flow.hooks.get(&hook_type).cloned() {
                self.push(hook, Some(command_id.clone()));
            }
        }
    }
}

impl <'a> Iterator for FlowIterator<'a> {
    type Item = (CommandId, Command, Option<CommandId>);

    /// Returns the next command to execute
    fn next(&mut self) -> Option<Self::Item> {
        match self.head() {
            // Empty Stack, we are done
            None => None,
            // We are at the end the current routine
            Some(item) if item.position == item.length => {
                self.pop();
                if !item.is_hook {
                    // We are at the end of the main routine, schedule the after job hook
                    if let Some(hook_id) = self.flow.hooks.get(&Hook::AfterJob) {
                        self.push(hook_id.clone(), None);
                    }
                }
                self.next()
            },
            // We are at the start of the current routine
            Some(item) if item.position == -1 => {
                self.increment_position();
                if !item.is_hook {
                    // We are at the start of the main routine, schedule the before job hook
                    if let Some(hook_id) = self.flow.hooks.get(&Hook::BeforeJob) {
                        self.push(hook_id.clone(), None);
                    }
                }
                self.next()
            }
            // We at a main routine command (non hook) which has not been scheduled
            Some(item) if !item.is_hook && !item.scheduled => {
                self.set_scheduled();
                // Add a task-specific hook
                let (command_id, command) = self.get_command(&item).expect("Command not found");
                let command_id = command_id.clone();
                if let Some(hook_id) = command.hooks.get(&Hook::BeforeTask).cloned() {
                    self.push(hook_id.clone(), Some(command_id.clone()));
                }
                // Add a global hook
                if let Some(hook_id) = self.flow.hooks.get(&Hook::BeforeTask) {
                    self.push(hook_id.clone(), Some(command_id.clone()));
                }
                self.next()
            }
            // We are at a command which has been scheduled
            Some(item) => {
                self.increment_position();

                let (command_id, command) = self.get_command(&item).expect("Command not found");
                let (command_id, command) = (command_id.clone(), command.clone());

                if !item.is_hook {
                    // Add a global hook
                    if let Some(hook_id) = self.flow.hooks.get(&Hook::AfterTask) {
                        self.push(hook_id.clone(), Some(command_id.clone()));
                    }
                    // Add a task-specific hook
                    if let Some(hook_id) = command.hooks.get(&Hook::AfterTask).cloned() {
                        self.push(hook_id.clone(), Some(command_id.clone()));
                    }
                }

                Some((command_id, command, item.focus_command.clone()))
            }
        }
    }
}