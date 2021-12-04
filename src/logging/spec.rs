use std::path::PathBuf;
use crate::config::{LoggingConfig, LogHandler, LogHandlerType};
use crate::execution::{ExecutionContext, resolve_cwd};
use crate::logging::OutputStream;
use anyhow::{Context as AnyhowContext, format_err, Result};


#[derive(Debug, Clone, Copy)]
pub enum InputStream {
    Stdout,
    Stderr,
    Both,
}

impl InputStream {
    pub fn is_stdout(&self) -> bool {
        match self {
            InputStream::Stderr => false,
            _ => true,
        }
    }

    pub fn is_stderr(&self) -> bool {
        match self {
            InputStream::Stdout => false,
            _ => true,
        }
    }

    pub fn is_compatible(&self, other: Self) -> bool {
        match (self, other) {
            (InputStream::Both, _) => true,
            (InputStream::Stdout, InputStream::Stdout) => true,
            (InputStream::Stderr, InputStream::Stderr) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileOutputSpec {
    pub file: PathBuf,
    pub append: bool,
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
    pub fn from_handler(handler: &LogHandler, input: InputStream, context: &ExecutionContext) -> Result<Vec<Self>> {
        if !handler.options.hooks && context.is_in_hook() {
            return Ok(Vec::new());
        }

        match &handler.handler {
            LogHandlerType::File(f) => {
                let mut file = resolve_cwd(&context.cwd, f.output.as_ref());
                if f.split {
                    if file.is_file() {
                        return Err(format_err!("Cannot create directory '{}'", file.display()));
                    }
                    std::fs::create_dir_all(&file)?;
                    let filename = if context.is_in_hook() {
                        context.focus.clone().unwrap_or_else(|| "job".to_string())
                    } else {
                        context.current_command_id().clone()
                    };
                    file.push(format!("{}.log", filename));
                }

                return Ok(vec![Self {
                    input,
                    output: OutputStreamSpec::File(FileOutputSpec {
                        file,
                        append: true,
                    }),
                }]);
            },
            LogHandlerType::Console => {
                let mut result = Vec::new();

                if input.is_stdout() {
                    result.push(Self {
                        input: InputStream::Stdout,
                        output: OutputStreamSpec::Stdout,
                    });
                }

                if input.is_stderr() {
                    result.push(Self {
                        input: InputStream::Stderr,
                        output: OutputStreamSpec::Stderr,
                    });
                }

                return Ok(result);
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoggingSpec {
    pub pipes: Vec<PipeSpec>,
}

impl LoggingSpec {
    pub fn from_config(config: &LoggingConfig, context: &ExecutionContext) -> Result<Self> {
        let mut result = Vec::new();
        for handler in &config.handlers {
            let input_stream = if handler.options.stdout && handler.options.stderr {
                InputStream::Both
            } else if handler.options.stdout {
                InputStream::Stdout
            } else if handler.options.stderr {
                InputStream::Stderr
            } else {
                continue;
            };

            result.extend(PipeSpec::from_handler(handler, input_stream, context)?);
        }

        Ok(Self {
            pipes: result,
        })
    }
}

