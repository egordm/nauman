use std::{
    fs,
    io::{self, BufWriter, Write},
    sync::{Mutex},
};
use std::ops::Index;

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

pub enum Output {
    Stdout(Stdout),
    Stderr(Stderr),
    File(File),
    Writer(Writer),
    Null(Null),
}

impl Output {
    pub fn new_stdout() -> Self {
        Output::Stdout(Stdout {
            stream: io::stdout(),
        })
    }

    pub fn new_stderr() -> Self {
        Output::Stderr(Stderr {
            stream: io::stderr(),
        })
    }

    pub fn new_file(path: &str) -> Self {
        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        Output::File(File {
            stream: Mutex::new(BufWriter::new(file)),
        })
    }

    pub fn new_writer(stream: Box<dyn Write + Send>) -> Self {
        Output::Writer(Writer {
            stream: Mutex::new(stream),
        })
    }

    pub fn new_null() -> Self {
        Output::Null(Null)
    }
}

impl std::io::Write for Output {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Output::Stdout(ref mut stdout) => stdout.write(buf),
            Output::Stderr(ref mut stderr) => stderr.write(buf),
            Output::File(ref mut file) => file.write(buf),
            Output::Writer(ref mut writer) => writer.write(buf),
            Output::Null(ref mut null) => null.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Output::Stdout(ref mut stdout) => stdout.flush(),
            Output::Stderr(ref mut stderr) => stderr.flush(),
            Output::File(ref mut file) => file.flush(),
            Output::Writer(ref mut writer) => writer.flush(),
            Output::Null(ref mut null) => null.flush(),
        }
    }
}

pub struct MultiplexedOutput {
    outputs: Vec<Output>,
}

impl Into<Output> for MultiplexedOutput {
    fn into(self) -> Output {
        Output::new_writer(Box::new(self))
    }
}

impl MultiplexedOutput {
    pub fn new() -> Self {
        MultiplexedOutput { outputs: Vec::new() }
    }

    pub fn add(&mut self, output: Output) {
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

pub struct DualOutput {
    pub stdout: Output,
    pub stderr: Output,
}

impl DualOutput {
    pub fn new(stdout: Output, stderr: Output) -> Self {
        DualOutput { stdout, stderr }
    }
}

impl DualWriter for DualOutput {
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
