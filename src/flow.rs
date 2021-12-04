use std::collections::HashMap;
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use crate::{
    common::Env,
    config,
    config::{Hook, LoggingConfig}
};
use crate::config::ExecutionPolicy;
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

fn format_identifier(text: &str) -> String {
    IDENTIFIER_REGEX.replace_all(
        &text.to_lowercase().replace(" ", "-"), ""
    ).to_string()
}

fn generate_id(name: &str, counter: usize, prefix: &str) -> String {
    format!("{}{:03}_{}", prefix, counter, format_identifier(name))
}

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub run: String,
    pub env: Env,
    pub cwd: Option<String>,
    pub is_hook: bool,
    pub hooks: Hooks,
    pub policy: ExecutionPolicy,
}

#[derive(Debug)]
pub struct Routine {
    commands: Vec<CommandId>,
    pub is_hook: bool,
}

#[derive(Debug)]
pub struct Flow {
    pub dependencies: Dependencies,
    pub routines: Routines,
    pub hooks: Hooks,
    pub env: Env,
    pub cwd: Option<String>,
}

impl Flow {
    pub fn parse(job: &config::Job) -> Result<Self> {
        FlowBuilder::new().parse_flow(job)
    }

    pub fn iter(&self) -> FlowIterator {
        FlowIterator::new(self)
    }
}

#[derive(Debug)]
pub struct FlowBuilder {
    pub dependencies: Dependencies,
    pub routines: Routines,
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

    pub fn parse_routine(
        &mut self,
        tasks: &config::Tasks,
        prefix: &str,
        is_hook: bool,
    ) -> Result<Routine> {
        let mut counter = 0;
        let mut commands = Vec::new();

        for task in tasks {
            let task_id = task.id.clone()
                .unwrap_or_else(|| generate_id(&task.name, counter, prefix));

            // TODO: handle this in a more verbose way
            let command = self.parse_command(task, &task_id, is_hook)?;
            if let Some(_) = self.dependencies.insert(task_id.clone(), command) {
                return Err(anyhow!("Task {} has a duplicate id", task.name));
            }

            commands.push(task_id);
            counter += 1;
        }

        Ok(Routine { commands,  is_hook })
    }

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
            name: task.name.clone(),
            run: task.run.clone(),
            env: task.env.clone().unwrap_or(Env::default()),
            cwd: task.cwd.clone(),
            is_hook,
            hooks,
            policy: task.policy.unwrap_or(self.policy),
        })
    }

    pub fn parse_flow(
        mut self,
        job: &config::Job,
    ) -> Result<Flow> {
        self.policy = job.policy;
        let hooks = self.parse_hooks(&job.hooks, "", false)?;
        let main_routine = self.parse_routine(&job.tasks, "", false)?;
        self.routines.insert(MAIN_ROUTINE_NAME.to_string(), main_routine);

        Ok(Flow {
            dependencies: self.dependencies,
            routines: self.routines,
            env: job.env.clone().unwrap_or_default(),
            cwd: job.cwd.clone(),
            hooks,
        })
    }
}


#[derive(Debug, Clone)]
pub struct StackItem {
    pub routine: RoutineId,
    pub position: i32,
    pub scheduled: bool,
    pub is_hook: bool,
    pub length: i32,
}

pub struct FlowIterator<'a> {
    flow: &'a Flow,
    routine_stack: Vec<StackItem>,
}

impl <'a> FlowIterator<'a> {
    pub fn new(flow: &'a Flow) -> Self {
        let mut res = FlowIterator { flow, routine_stack: Vec::new() };
        res.push(MAIN_ROUTINE_NAME.to_string()  );
        res
    }

    fn head(&self) -> Option<StackItem> {
        self.routine_stack.last().cloned()
    }

    fn head_mut(&mut self) -> Option<&mut StackItem> {
        self.routine_stack.last_mut()
    }

    fn increment_position(&mut self) {
        if let Some(item) = self.head_mut() {
            item.position += 1;
            item.scheduled = false;
        }
    }

    fn set_scheduled(&mut self) {
        if let Some(item) = self.head_mut() {
            item.scheduled = true;
        }
    }

    fn routine(&self, routine_id: &RoutineId) -> &Routine {
        self.flow.routines.get(routine_id).expect("Routine not found")
    }

    fn command(&self, command_id: &CommandId) -> &Command {
        self.flow.dependencies.get(command_id).expect("Command not found")
    }

    fn get_command(&self, head: &StackItem) -> Option<(&CommandId, &Command)> {
        let routine = self.routine(&head.routine);
        let command_id = routine.commands.get(head.position as usize).expect("Command not found");
        let command = self.command(&command_id);
        Some((command_id, command))
    }

    fn push(&mut self, routine_id: RoutineId) {
        let routine = self.routine(&routine_id);

        self.routine_stack.push(StackItem{
            routine: routine_id,
            position: -1,
            scheduled: false,
            is_hook: routine.is_hook,
            length: routine.commands.len() as i32,
        });
    }

    fn pop(&mut self) {
        self.routine_stack.pop();
    }

    pub fn push_result(
        &mut self,
        command_id: &CommandId,
        result: &ExecutionResult
    ) {
        let hook_type = if result.is_success() { Hook::OnSuccess } else { Hook::OnFailure };

        let command = self.command(command_id);
        if !command.is_hook {
            // Add a task-specific hook
            if let Some(hook) = command.hooks.get(&hook_type) {
                self.push(hook.clone());
            }
            // Add a global hook
            if let Some(hook) = self.flow.hooks.get(&hook_type) {
                self.push(hook.clone());
            }
        }
    }

}

impl <'a> Iterator for FlowIterator<'a> {
    type Item = (CommandId, Command);

    fn next(&mut self) -> Option<Self::Item> {
        match self.head() {
            // Empty Stack, we are done
            None => None,
            Some(item) if item.position == item.length => {
                self.pop();
                if !item.is_hook {
                    if let Some(hook_id) = self.flow.hooks.get(&Hook::AfterJob) {
                        self.push(hook_id.clone());
                    }
                }
                self.next()
            },
            Some(item) if item.position == -1 => {
                self.increment_position();
                if !item.is_hook {
                    if let Some(hook_id) = self.flow.hooks.get(&Hook::BeforeJob) {
                        self.push(hook_id.clone());
                    }
                }
                self.next()
            }
            Some(item) if !item.is_hook && !item.scheduled => {
                self.set_scheduled();
                // Add a task-specific hook
                let (_, command) = self.get_command(&item).expect("Command not found");
                if let Some(hook_id) = command.hooks.get(&Hook::BeforeTask).cloned() {
                    self.push(hook_id.clone());
                }
                // Add a global hook
                if let Some(hook_id) = self.flow.hooks.get(&Hook::BeforeTask) {
                    self.push(hook_id.clone());
                }
                self.next()
            }
            Some(item) => {
                self.increment_position();

                let (command_id, command) = self.get_command(&item).expect("Command not found");
                let (command_id, command) = (command_id.clone(), command.clone());

                if !item.is_hook {
                    // Add a global hook
                    if let Some(hook_id) = self.flow.hooks.get(&Hook::AfterTask) {
                        self.push(hook_id.clone());
                    }
                    // Add a task-specific hook
                    if let Some(hook_id) = command.hooks.get(&Hook::AfterTask).cloned() {
                        self.push(hook_id.clone());
                    }
                }

                Some((command_id, command))
            }
            _ => None
        }
    }
}