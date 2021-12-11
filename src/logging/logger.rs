use std::io;
use std::io::Write;
use std::path::PathBuf;
use crate::{execution::ExecutionContext, config::{LogHandlers, Shell}, logging::{MultiOutputStream, LoggingSpec, pprint}, common::Env, flow::Command, flow};
use anyhow::{Result};
use colored::{Colorize};
use prettytable::{Cell, row, Row, Table};
use crate::common::LogLevel;
use crate::execution::ExecutionResult;
use crate::pprint::{flex_banner, truncate_string};

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

pub struct ActionSummary<'a> {
    pub flow: &'a flow::Flow,
    pub results: &'a Vec<(String, ExecutionResult)>,
}

impl<'a> LogAction for ActionSummary<'a> {
    fn min_level(&self) -> LogLevel {
        LogLevel::Info
    }

    fn write(&self, _level: LogLevel, _output: &mut impl Write) -> std::io::Result<()> {
        let mut table = Table::new();
        table.add_row(row![
            "Task",
            "Action",
            "Time (in s)",
        ]);

        for (command_id, result) in self.results.iter() {
            let command = self.flow.command(command_id).expect("Command not found");
            let step = if result.is_success() {
                if command.is_hook { "🪝".to_string() } else { command.task_no.map(|i| i.to_string()).unwrap_or_default() }
            } else if result.is_aborted() { "⛔️".to_string() } else { "💥".to_string() };
            let name = &truncate_string(&command.name, 60);
            let duration = result.duration.map(|d| d.as_secs().to_string())
                .unwrap_or_else(|| "-".to_string());

            table.add_row(Row::new(vec![
                Cell::new(&step),
                Cell::new(name).style_spec(if !result.is_success() { "Fr" } else { "" }),
                Cell::new(&duration),
            ]));
        }

        println!("{}", flex_banner(format!("Summary: {}", self.flow.name)).yellow());
        table.printstd();
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
        let spec = LoggingSpec::from_config(&self.config, context)?;
        self.output = MultiOutputStream::from_spec(spec);
        Ok(())
    }

    pub fn mut_output(&mut self) -> &mut MultiOutputStream {
        &mut self.output
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }

    pub fn log_action(&mut self, action: impl LogAction) -> Result<()> {
        if self.level >= action.min_level() {
            action.write(self.level, self.mut_output())?;
        }
        Ok(())
    }
}