use crate::config::LoggingConfig;
use crate::execution::ExecutionContext;
use crate::logging::{DualOutputStream, LoggingSpec};
use anyhow::{Context as AnyhowContext, Result};
use crate::flow::Command;


pub struct Logger {
    pub config: LoggingConfig,
    pub output: DualOutputStream,
}

impl Logger {
    pub fn new(config: LoggingConfig) -> Logger {
        Logger {
            config,
            output: DualOutputStream::new(),
        }
    }

    pub fn switch(
        &mut self,
        _command: &Command,
        context: &ExecutionContext,
    ) -> Result<()> {
        let spec = LoggingSpec::from_config(&self.config, &context)?;
        self.output = DualOutputStream::from_spec(spec);
        Ok(())
    }

    pub fn mut_output(&mut self) -> &mut DualOutputStream {
        &mut self.output
    }
}