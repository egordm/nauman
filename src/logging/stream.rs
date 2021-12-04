use std::{
    fs,
    io::{self, BufWriter, Write},
    sync::{Mutex},
    ops::Index,
    path::Path,
};
use crate::config::{LoggingConfig, LogHandler, LogHandlerType};
use crate::logging::{InputStream, OutputStreamSpec, PipeSpec};

pub struct Stdout {
    pub stream: io::Stdout,
}

pub struct Stderr {
    pub stream: io::Stderr,
}

pub struct File {
    pub stream: Mutex<BufWriter<fs::File>>,
}

pub struct Writer {
    pub stream: Mutex<Box<dyn Write + Send>>,
}

pub struct Null;

impl std::io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

impl std::io::Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

impl std::io::Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.lock().unwrap().flush()
    }
}

impl std::io::Write for Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.lock().unwrap().flush()
    }
}

impl std::io::Write for Null {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Ok(0)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub enum OutputStream {
    Stdout(Stdout),
    Stderr(Stderr),
    File(File),
    Writer(Writer),
    Null(Null),
}

impl OutputStream {
    pub fn new_stdout() -> Self {
        OutputStream::Stdout(Stdout {
            stream: io::stdout(),
        })
    }

    pub fn new_stderr() -> Self {
        OutputStream::Stderr(Stderr {
            stream: io::stderr(),
        })
    }

    pub fn new_file(path: impl AsRef<Path>, append: bool) -> Self {
        let file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(append)
            .open(path)
            .unwrap(); // TODO: handle error
        OutputStream::File(File {
            stream: Mutex::new(BufWriter::new(file)),
        })
    }

    pub fn new_writer(stream: Box<dyn Write + Send>) -> Self {
        OutputStream::Writer(Writer {
            stream: Mutex::new(stream),
        })
    }

    pub fn new_null() -> Self {
        OutputStream::Null(Null)
    }
}

impl std::io::Write for OutputStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            OutputStream::Stdout(ref mut stdout) => stdout.write(buf),
            OutputStream::Stderr(ref mut stderr) => stderr.write(buf),
            OutputStream::File(ref mut file) => file.write(buf),
            OutputStream::Writer(ref mut writer) => writer.write(buf),
            OutputStream::Null(ref mut null) => null.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            OutputStream::Stdout(ref mut stdout) => stdout.flush(),
            OutputStream::Stderr(ref mut stderr) => stderr.flush(),
            OutputStream::File(ref mut file) => file.flush(),
            OutputStream::Writer(ref mut writer) => writer.flush(),
            OutputStream::Null(ref mut null) => null.flush(),
        }
    }
}

impl From<OutputStreamSpec> for OutputStream {
    fn from(spec: OutputStreamSpec) -> Self {
        match spec {
            OutputStreamSpec::Stdout => OutputStream::new_stdout(),
            OutputStreamSpec::Stderr => OutputStream::new_stderr(),
            OutputStreamSpec::File(f) => OutputStream::new_file(
                f.file, f.append && !f.create,
            )
        }
    }
}

pub struct MultiplexedOutput {
    outputs: Vec<OutputStream>,
}

impl Into<OutputStream> for MultiplexedOutput {
    fn into(self) -> OutputStream {
        OutputStream::new_writer(Box::new(self))
    }
}

impl MultiplexedOutput {
    pub fn new() -> Self {
        MultiplexedOutput { outputs: Vec::new() }
    }

    pub fn add(&mut self, output: OutputStream) {
        self.outputs.push(output);
    }
}

impl std::io::Write for MultiplexedOutput {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for output in &mut self.outputs {
            output.write(buf)?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        for output in &mut self.outputs {
            output.flush()?;
        }
        Ok(())
    }
}

pub trait DualWriter {
    fn write_stdout(&mut self, buf: &[u8]) -> io::Result<usize>;
    fn write_stderr(&mut self, buf: &[u8]) -> io::Result<usize>;

    fn flush_stdout(&mut self) -> io::Result<()>;
    fn flush_stderr(&mut self) -> io::Result<()>;
}

pub struct DualOutputStream {
    pub stdout: OutputStream,
    pub stderr: OutputStream,
}

impl DualOutputStream {
    pub fn new(stdout: OutputStream, stderr: OutputStream) -> Self {
        DualOutputStream { stdout, stderr }
    }

    pub fn from_spec(specs: Vec<PipeSpec>) -> Self {
        let mut stdout = MultiplexedOutput::new();
        let mut stderr = MultiplexedOutput::new();

        for PipeSpec { output, input } in specs {
            let stream = output.into();
            match input {
                InputStream::Stdout => {
                    stdout.add(stream);
                }
                InputStream::Stderr => {
                    stderr.add(stream);
                }
            }
        }

        Self::new(stdout.into(), stderr.into())
    }
}

impl DualWriter for DualOutputStream {
    fn write_stdout(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdout.write(buf)
    }

    fn write_stderr(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdout.write(buf)
    }

    fn flush_stdout(&mut self) -> io::Result<()> {
        self.stdout.flush()
    }

    fn flush_stderr(&mut self) -> io::Result<()> {
        self.stderr.flush()
    }
}
