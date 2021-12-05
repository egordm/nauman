use std::io::Write;
use std::path::PathBuf;
use crate::{
    execution::ExecutionContext,
    config::{LogHandlers, Shell},
    logging::{MultiOutputStream, LoggingSpec, pprint},
    common::Env,
    flow::Command,
    LogLevel
};
use anyhow::{Result};
use colored::{Colorize};
use crate::config::ShellType;
use crate::execution::ExecutionResult;

pub trait LogAction {
    fn min_level(&self) -> LogLevel;

    fn write(&self, level: LogLevel, output: &mut impl std::io::Write) -> std::io::Result<()>;
}

pub struct ActionCommandStart<'a> {
    pub command: &'a Command
}

impl<'a> LogAction for ActionCommandStart<'a> {
    fn min_level(&self) -> LogLevel {
        LogLevel::Info
    }

    fn write(&self, _level: LogLevel, output: &mut impl std::io::Write) -> std::io::Result<()> {
        writeln!(output, "{}", if self.command.is_hook {
            pprint::flex_banner(format!("Hook: {}", &self.command.name)).yellow()
        } else {
            pprint::flex_banner( format!("Task: {}", &self.command.name)).green()
        })
    }
}

pub struct ActionShell<'a> {
    pub handler: &'a Shell,
    pub cwd: &'a PathBuf,
    pub env: &'a Env,
    pub shell_program: &'a str,
    pub shell_args: &'a [String],
}

impl<'a> LogAction for ActionShell<'a> {
    fn min_level(&self) -> LogLevel {
        LogLevel::Info
    }

    fn write(&self, level: LogLevel, output: &mut impl std::io::Write) -> std::io::Result<()> {
        if level >= LogLevel::Info {
            writeln!(output, "{}", pprint::command(&self.handler.run))?;
        }
        Ok(())
    }
}

pub struct ActionCommandEnd<'a> {
    pub command: &'a Command,
    pub result: &'a ExecutionResult,
}

impl<'a> LogAction for ActionCommandEnd<'a> {
    fn min_level(&self) -> LogLevel {
        LogLevel::Error
    }

    fn write(&self, level: LogLevel, output: &mut impl Write) -> std::io::Result<()> {
        if !self.result.is_success() && !self.result.is_aborted() && level >= LogLevel::Error {
            writeln!(output, "{}", pprint::task_error(
                &self.command.name, self.result.exit_code, self.result.duration.as_ref()
            ))?;
        }
        if self.result.is_aborted() && level >= LogLevel::Debug {
            if !self.command.is_hook {
                writeln!(output, "{}", pprint::task_aborted(
                    &self.command.name, self.command.policy
                ))?;
            }
        }
        if self.result.is_success() && !self.result.is_aborted() && level >= LogLevel::Debug {
            writeln!(output, "{}", pprint::task_success(
                &self.command.name, self.result.duration.as_ref()
            ))?;
        }
        Ok(())
    }
}

pub struct Logger {
    config: LogHandlers,
    level: LogLevel,
    pub output: MultiOutputStream,
}

impl Logger {
    pub fn new(config: LogHandlers, level: LogLevel) -> Logger {
        Logger {
            config,
            level,
            output: MultiOutputStream::new(),
        }
    }

    pub fn switch(
        &mut self,
        context: &ExecutionContext,
    ) -> Result<()> {
        let spec = LoggingSpec::from_config(&self.config, &context)?;
        self.output = MultiOutputStream::from_spec(spec);
        Ok(())
    }

    pub fn mut_output(&mut self) -> &mut MultiOutputStream {
        &mut self.output
    }

    pub fn log_action(&mut self, action: impl LogAction) -> Result<()> {
        if self.level >= action.min_level() {
            action.write(self.level, self.mut_output())?;
        }
        Ok(())
    }
}