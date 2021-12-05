use std::path::PathBuf;
use crate::config::{LoggingConfig, LogLevel, Shell};
use crate::execution::ExecutionContext;
use crate::logging::{MultiOutputStream, LoggingSpec, pprint};
use anyhow::{Context as AnyhowContext, Result};
use colored::{ColoredString, Colorize};
use crate::common::Env;
use crate::flow::Command;

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
    pub shell: &'a str,
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

pub struct Logger {
    config: LoggingConfig,
    pub output: MultiOutputStream,
}

impl Logger {
    pub fn new(config: LoggingConfig) -> Logger {
        colored::control::set_override(config.ansi);
        Logger {
            config,
            output: MultiOutputStream::new(),
        }
    }

    pub fn switch(
        &mut self,
        command: &Command,
        context: &ExecutionContext,
    ) -> Result<()> {
        let spec = LoggingSpec::from_config(&self.config, &context)?;
        self.output = MultiOutputStream::from_spec(spec);
        self.log_action(ActionCommandStart { command })?;
        Ok(())
    }

    pub fn mut_output(&mut self) -> &mut MultiOutputStream {
        &mut self.output
    }

    pub fn log_action(&mut self, action: impl LogAction) -> Result<()> {
        if action.min_level() >= self.level() {
            action.write(self.level(), self.mut_output())?;
        }
        Ok(())
    }

    pub fn level(&self) -> LogLevel {
        self.config.level
    }

    pub fn get_config(&self) -> &LoggingConfig {
        &self.config
    }
}