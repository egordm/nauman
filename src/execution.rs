use std::{
    io,
    fs::File,
    io::{BufReader, Read, Write},
    path::{PathBuf},
    process::Stdio,
    os::unix::io::{AsRawFd, FromRawFd},
};
use crate::{flow, flow::CommandId, logging::{OutputStream, MultiOutputStream, MultiWriter}, common::Env, logging, Logger};
use anyhow::{Context as AnyhowContext, Result};
use crate::config::{ExecutionPolicy, LoggingConfig, Shell, TaskHandler};
use crate::logging::{ActionShell, InputStream, LoggingSpec, PipeSpec};
use colored::*;
use crate::flow::Command;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ExecutionState {
    Running,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub command_id: CommandId,
    pub exit_code: i32,
    pub aborted: bool,
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
    pub env: Env,
    pub cwd: PathBuf,
    pub state: ExecutionState,
    pub will_execute: bool,
    pub current: Option<(CommandId, Command)>,
    pub focus: Option<CommandId>,
    pub previous: Option<ExecutionResult>,
}

impl ExecutionContext {
    pub fn new(env: Env, cwd: PathBuf) -> Self {
        Self {
            env,
            cwd,
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

    pub fn current_command(&self) -> &Command {
        self.current.as_ref().map(|(_, command)| command).expect("No current command")
    }
}

pub fn resolve_cwd(current: &PathBuf, cwd: Option<&String>) -> PathBuf {
    let cwd = cwd.map(PathBuf::from).unwrap_or_else(|| current.clone());
    if cwd.is_absolute() {
        cwd
    } else {
        current.join(cwd)
    }
}

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
        let shell = "sh";

        // Announce execution
        logger.log_action(ActionShell {
            handler: &self,
            env: &env,
            cwd: &cwd,
            shell: &shell,
        })?;

        // Build command
        let mut child = std::process::Command::new(shell)
            .args(&["-c", &self.run])
            .envs(env)
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to execute command: {}", self.run))?;

        // Execute command, capture its output and return its exit code
        capture_command(&child, logger.mut_output())?;
        let exit_code = child.wait()?.code().unwrap_or(-1);

        Ok(ExecutionResult {
            command_id: context.current_command_id().clone(),
            exit_code,
            aborted: false,
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

pub const ENV_PREV_CODE: &str = "NAUMAN_PREV_CODE";
pub const ENV_PREV_ID: &str = "NAUMAN_PREV_ID";

pub struct Executor<'a> {
    pub flow: &'a flow::Flow,
    pub context: ExecutionContext,
}

impl<'a> Executor<'a> {
    pub fn new(flow: &'a flow::Flow) -> Result<Self> {
        let mut env: Env = std::env::vars().collect();
        env.extend(flow.env.clone());

        let cwd = resolve_cwd(&std::env::current_dir()?, flow.cwd.as_ref());

        Ok(Executor {
            flow,
            context: ExecutionContext::new(env, cwd),
        })
    }

    pub fn execute(
        &mut self,
        logger: &mut Logger,
    ) -> Result<()> {
        let mut results = Vec::new();
        let mut flow_iter = self.flow.iter();
        while let Some((command_id, command, focus_id)) = flow_iter.next() {
            let result = self.execute_step(&command_id, &command, focus_id.as_ref(), logger)?;
            flow_iter.push_result(&command_id, &result);

            if !command.is_hook {
                results.push((command_id.clone(), result));
            }
        }
        Ok(())
    }

    pub fn execute_step(
        &mut self,
        command_id: &CommandId,
        command: &flow::Command,
        focus_id: Option<&CommandId>,
        logger: &mut Logger,
    ) -> Result<ExecutionResult> {
        self.context.current = Some((command_id.clone(), command.clone()));
        self.context.focus = focus_id.map(|id| id.clone());
        self.context.will_execute = match command.policy {
            ExecutionPolicy::NoPriorFailed => self.context.state != ExecutionState::Failed,
            ExecutionPolicy::PriorSuccess => self.context.previous.as_ref().map(|r| r.is_success()).unwrap_or(true),
            ExecutionPolicy::Always => true
        };
        logger.switch(command, &self.context)?;

        let result = if self.context.will_execute {
            // Prepare context
            if let Some(previous) = self.context.previous.as_ref() {
                self.context.env.insert(ENV_PREV_CODE.to_string(), previous.exit_code.to_string());
                self.context.env.insert(ENV_PREV_ID.to_string(), previous.command_id.to_string());
            }

            execute_command(command, &mut self.context, logger)?
        } else {
            ExecutionResult {
                command_id: command_id.clone(),
                exit_code: 0,
                aborted: true,
            }
        };

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