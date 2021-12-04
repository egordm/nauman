use std::{
    fs,
    io::{self, BufWriter, Write},
    sync::{Mutex},
    ops::Index,
    path::Path,
};
use std::cell::RefCell;
use std::rc::Rc;
use crate::config::{LoggingConfig, LogHandler, LogHandlerType};
use crate::logging::{InputStream, LoggingSpec, OutputStreamSpec, PipeSpec};

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

pub struct SharedWriter {
    pub stream: Rc<RefCell<dyn Write + Send>>,
}

impl SharedWriter {
    pub fn split(self) -> (Self, Self) {
        (
            SharedWriter {
                stream: self.stream.clone(),
            },
            SharedWriter {
                stream: self.stream,
            },
        )
    }
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
                f.file, f.append,
            )
        }
    }
}

pub trait MultiWriter {
    fn write_stream(&mut self, stream: InputStream, buf: &[u8]) -> io::Result<usize>;

    fn flush_stream(&mut self, stream: InputStream) -> io::Result<()>;
}

pub struct MultiOutputStream {
    pub outputs: Vec<(InputStream, OutputStream)>,
}

impl MultiOutputStream {
    pub fn new() -> Self {
        MultiOutputStream { outputs: Vec::new() }
    }

    pub fn from_spec(specs: LoggingSpec) -> Self {
        let mut outputs = Vec::new();

        for PipeSpec { output, input } in specs.pipes {
            outputs.push((input, output.into()));
        }

        Self { outputs }
    }
}

impl std::io::Write for MultiOutputStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        MultiWriter::write_stream(self, InputStream::Stdout, buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        MultiWriter::flush_stream(self, InputStream::Stdout)
    }
}

impl MultiWriter for MultiOutputStream {
    fn write_stream(&mut self, stream: InputStream, buf: &[u8]) -> io::Result<usize> {
        let mut written = 0;
        for (input, output) in &mut self.outputs {
            if input.is_compatible(stream) {
                written = output.write(buf)?.max(written);
            }
        }
        Ok(written)
    }

    fn flush_stream(&mut self, stream: InputStream) -> io::Result<()> {
        for (input, output) in &mut self.outputs {
            if input.is_compatible(stream) {
                output.flush()?;
            }
        }
        Ok(())
    }
}
