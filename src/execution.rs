use std::{
    io,
    fs::File,
    io::{BufReader, Read},
    path::{PathBuf},
    process::Stdio,
    os::unix::io::{AsRawFd, FromRawFd},
};
use std::collections::HashMap;
use std::time::Instant;
use crate::{
    flow,
    flow::CommandId,
    logging::{MultiOutputStream, MultiWriter},
    common::Env,
    config,
    config::{ExecutionPolicy, Shell, TaskHandler},
    logging::{ActionShell, InputStream},
    flow::Command,
    logging::ActionCommandStart
};
use anyhow::{Context as AnyhowContext, Result};
use chrono::{Local};
use crate::logging::{ActionCommandEnd, Logger};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ExecutionState {
    Running,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub command_id: CommandId,
    pub focus_id: Option<CommandId>,
    pub exit_code: i32,
    pub aborted: bool,
    pub duration: Option<std::time::Duration>,
}

impl ExecutionResult {
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }

    pub fn is_aborted(&self) -> bool {
        self.aborted
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub options: config::Options,
    pub env: Env,
    pub cwd: PathBuf,
    pub log_dir: PathBuf,
    pub state: ExecutionState,
    pub will_execute: bool,
    pub current: Option<(CommandId, Command)>,
    pub focus: Option<CommandId>,
    pub previous: Option<ExecutionResult>,
}

impl ExecutionContext {
    pub fn new(options: config::Options, env: Env, cwd: PathBuf) -> Self {
        Self {
            options,
            cwd,
            env: Env::default(),
            log_dir: PathBuf::new(),
            state: ExecutionState::Running,
            will_execute: true,
            current: None,
            focus: None,
            previous: None,
        }
    }

    pub fn is_in_hook(&self) -> bool {
        if let Some((_, command)) = &self.current {
            command.is_hook
        } else {
            false
        }
    }

    pub fn current_command_id(&self) -> &CommandId {
        self.current.as_ref().map(|(id, _)| id).expect("No current command")
    }
}

/// Returns cwd path relative to the current cwd.
/// If the desired cwd is absolute, it is returned as is.
pub fn resolve_cwd(current: &PathBuf, cwd: Option<&String>) -> PathBuf {
    let cwd = cwd.map(PathBuf::from).unwrap_or_else(|| current.clone());
    if cwd.is_absolute() {
        cwd
    } else {
        current.join(cwd)
    }
}

/// Reads the contents of a file into a buffer.
fn read_buffer(source: &mut BufReader<File>, buffer: &mut [u8]) -> io::Result<Option<usize>> {
    match source.read(buffer) {
        Ok(count) => Ok(Some(count)),
        Err(e) => match e.kind() {
            io::ErrorKind::WouldBlock => Ok(None),
            _ => Err(e),
        },
    }
}

const BUFFER_SIZE: usize = 1024; // 1 KB

/// Responsible for executing a command and capturing its output.
pub fn capture_command(
    child: &std::process::Child,
    output: &mut MultiOutputStream,
) -> Result<()> {
    // TODO: split into two functions
    let mut buffer = [0; BUFFER_SIZE];
    let (mut stdout_done, mut stderr_done) = (false, false);
    let mut stdout = BufReader::new(unsafe {
        File::from_raw_fd(child.stdout.as_ref().unwrap().as_raw_fd())
    });
    let mut stderr = BufReader::new(unsafe {
        File::from_raw_fd(child.stderr.as_ref().unwrap().as_raw_fd())
    });

    loop {
        match read_buffer(&mut stdout, &mut buffer) {
            Ok(None) => break,
            Ok(Some(size)) if size == 0 => {
                stdout_done = true;
            }
            Ok(Some(size)) => {
                output.write_stream(InputStream::Stdout, &buffer[0..size]).unwrap();
            }
            Err(e) => return Err(e.into()),
        }

        match read_buffer(&mut stderr, &mut buffer) {
            Ok(None) => break,
            Ok(Some(size)) if size == 0 => {
                stderr_done = true;
            }
            Ok(Some(size)) => {
                output.write_stream(InputStream::Stderr, &buffer[0..size]).unwrap();
            }
            Err(e) => return Err(e.into()),
        }

        if stderr_done && stdout_done {
            break;
        }
    }

    Ok(())
}

pub trait ExecutableHandler {
    fn execute(
        &self,
        command: &flow::Command,
        context: &mut ExecutionContext,
        logger: &mut Logger,
    ) -> Result<ExecutionResult>;
}

impl ExecutableHandler for Shell {
    fn execute(
        &self,
        command: &flow::Command,
        context: &mut ExecutionContext,
        logger: &mut Logger,
    ) -> Result<ExecutionResult> {
        // Build env
        let mut env = context.env.clone();
        env.extend(command.env.clone());

        // Build cwd
        let cwd = resolve_cwd(&context.cwd, command.cwd.as_ref());

        // Build shell
        let shell = self.shell.clone().unwrap_or_else(|| context.options.shell.clone());
        let shell_path = self.shell_path.as_ref().or_else(|| {
            if shell == context.options.shell {
                context.options.shell_path.as_ref()
            } else {
                None
            }
        });
        let shell_program = shell.executable(shell_path)?;
        let shell_args = shell.args(shell_path, self.run.clone())?;

        // Announce execution
        logger.log_action(ActionShell {
            handler: self,
            env: &env,
            cwd: &cwd,
            shell_program: &shell_program,
            shell_args: shell_args.as_slice(),
        })?;

        // Time execution
        let now = Instant::now();

        let exit_code = if context.options.dry_run {
            // Always succeed on dry run
            0
        } else {
            // Build command
            let mut child = std::process::Command::new(shell_program)
                .args(&shell_args)
                .envs(env)
                .current_dir(cwd)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .with_context(|| format!("Failed to execute command: {}", self.run))?;

            // Execute command, capture its output and return its exit code
            capture_command(&child, logger.mut_output())?;
            child.wait()?.code().unwrap_or(-1)
        };

        Ok(ExecutionResult {
            command_id: context.current_command_id().clone(),
            focus_id: context.focus.clone(),
            exit_code,
            aborted: false,
            duration: Some(now.elapsed()),
        })
    }
}

impl ExecutableHandler for TaskHandler {
    fn execute(&self, command: &Command, context: &mut ExecutionContext, logger: &mut Logger) -> Result<ExecutionResult> {
        match self {
            TaskHandler::Shell(handler) => handler.execute(command, context, logger),
        }
    }
}

pub fn execute_command(
    command: &flow::Command,
    context: &mut ExecutionContext,
    logger: &mut Logger,
) -> Result<ExecutionResult> {
    command.handler.execute(command, context, logger)
}

pub const ENV_PREV_NAME: &str = "NAUMAN_PREV_NAME";
pub const ENV_PREV_ID: &str = "NAUMAN_PREV_ID";
pub const ENV_PREV_CODE: &str = "NAUMAN_PREV_CODE";
pub const ENV_JOB_NAME: &str = "NAUMAN_JOB_NAME";
pub const ENV_JOB_ID: &str = "NAUMAN_JOB_ID";
pub const ENV_TASK_NAME: &str = "NAUMAN_TASK_NAME";
pub const ENV_TASK_ID: &str = "NAUMAN_TASK_ID";


/// Executor responsible for executing a flow.
pub struct Executor<'a> {
    pub flow: &'a flow::Flow,
    pub context: ExecutionContext,
}

impl<'a> Executor<'a> {
    pub fn new(
        options: config::Options,
        flow: &'a flow::Flow
    ) -> Result<Self> {
        let cwd = resolve_cwd(&std::env::current_dir()?, flow.cwd.as_ref());

        Ok(Executor {
            flow,
            context: ExecutionContext::new(options, env, cwd),
        })
    }

    /// Execute a whole flow
    pub fn execute(
        &mut self,
        logger: &mut Logger,
    ) -> Result<()> {
        // Create log dir
        self.context.log_dir = resolve_cwd(&self.context.cwd, self.context.options.log_dir.as_ref());
        self.context.log_dir.push(
            format!("{}_{}", self.flow.id, Local::now().format("%Y-%m-%dT%H:%M:%S"))
        );
        std::fs::create_dir_all(&self.context.log_dir)?;

        // Define global context variables
        self.context.env.insert(ENV_JOB_NAME.to_string(), self.flow.id.clone());
        self.context.env.insert(ENV_JOB_ID.to_string(), self.flow.id.clone());

        // Loop through all the commands and store the results
        let mut results = Vec::new();
        let mut flow_iter = self.flow.iter();
        while let Some((command_id, command, focus_id)) = flow_iter.next() {
            let result = self.execute_step(&command_id, &command, focus_id.as_ref(), logger)?;
            flow_iter.push_result(&command_id, &result);

            results.push((command_id.clone(), result));
        }
        Ok(())
    }

    /// Execute a single command
    pub fn execute_step(
        &mut self,
        command_id: &CommandId,
        command: &flow::Command,
        focus_id: Option<&CommandId>,
        logger: &mut Logger,
    ) -> Result<ExecutionResult> {
        // Set up the context for the current command
        self.context.current = Some((command_id.clone(), command.clone()));
        self.context.focus = focus_id.cloned();
        self.context.will_execute = match command.policy {
            ExecutionPolicy::NoPriorFailed => self.context.state != ExecutionState::Failed,
            ExecutionPolicy::PriorSuccess => self.context.previous.as_ref().map(|r| r.is_success()).unwrap_or(true),
            ExecutionPolicy::Always => true
        };

        // Switch logger context to the current command
        logger.switch(&self.context)?;

        // Execute the command if possible
        let result = if self.context.will_execute {
            // Announce command to execute
            logger.log_action(ActionCommandStart { command })?;

            // Prepare context
            // TODO: should this be moved to context preparation?
            if let Some(previous) = self.context.previous.as_ref() {
                let prev_command = self.flow.command(&previous.command_id).expect("Previous command not found");
                self.context.env.insert(ENV_PREV_CODE.to_string(), previous.exit_code.to_string());
                self.context.env.insert(ENV_PREV_ID.to_string(), previous.command_id.clone());
                self.context.env.insert(ENV_PREV_NAME.to_string(), prev_command.name.clone());
            }
            self.context.env.insert(ENV_TASK_NAME.to_string(), command.name.clone());
            self.context.env.insert(ENV_TASK_ID.to_string(), command_id.clone());

            // Execute the actual command
            execute_command(command, &mut self.context, logger)?
        } else {
            ExecutionResult {
                command_id: command_id.clone(),
                focus_id: focus_id.cloned(),
                exit_code: 0,
                aborted: true,
                duration: None,
            }
        };

        // Announce command result
        logger.log_action(ActionCommandEnd { command, result: &result })?;

        // Only main command state is stored
        if !command.is_hook {
            if !result.is_success() && !result.is_aborted() {
                self.context.state = ExecutionState::Failed;
            }

            self.context.previous = Some(result.clone());
        }
        Ok(result)
    }
}