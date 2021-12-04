use crate::config::{LoggingConfig, LogHandler, LogHandlerType};
use crate::logging::OutputStream;

#[derive(Debug, Clone, Copy)]
pub enum InputStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone)]
pub struct FileOutputSpec {
    pub file: String,
    pub append: bool,
    pub create: bool,
}

#[derive(Debug, Clone)]
pub enum OutputStreamSpec {
    Stdout,
    Stderr,
    File(FileOutputSpec),
}

#[derive(Debug, Clone)]
pub struct PipeSpec {
    pub input: InputStream,
    pub output: OutputStreamSpec,
}

impl PipeSpec {
    pub fn from_handler(handler: &LogHandler, input: InputStream) -> Self {
        match &handler.handler {
            LogHandlerType::File(f) => PipeSpec {
                input,
                output: OutputStreamSpec::File(FileOutputSpec {
                    file: f.output.clone().unwrap_or("stderr.log".to_string()),
                    append: !f.split,
                    create: true, // TODO: this is context based
                }),
            },
            LogHandlerType::Console => PipeSpec {
                input,
                output: match input {
                    InputStream::Stdout => OutputStreamSpec::Stdout,
                    InputStream::Stderr => OutputStreamSpec::Stderr,
                },
            },
        }
    }

    pub fn from_config(config: &LoggingConfig) -> Vec<Self> {
        let mut result = Vec::new();
        for handler in &config.handlers {
            if handler.options.stdout {
                result.push(PipeSpec::from_handler(handler, InputStream::Stdout));
            }
            if handler.options.stderr {
                result.push(PipeSpec::from_handler(handler, InputStream::Stderr));
            }
        }
        result
    }
}
