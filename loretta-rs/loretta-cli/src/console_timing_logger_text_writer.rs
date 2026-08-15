// Ported from Loretta.CLI.ConsoleTimingLoggerTextWriter (b767b4e): ConsoleTimingLoggerTextWriter
// C# source: src/Compilers/Lua/CommandLine/ConsoleTimingLoggerTextWriter.cs

use std::io::{self, Write};

/// Trait abstracting the external Tsu.Timing.ConsoleTimingLogger.
/// In C#, ConsoleTimingLogger provides Write/WriteLine methods with timing prefixes.
pub trait TimingLogger {
    fn write_str(&self, s: &str);
    fn write_char(&self, c: char);
    fn write_line(&self, s: &str);
}

/// Wraps a TimingLogger and implements std::io::Write,
/// mirroring C# ConsoleTimingLoggerTextWriter : TextWriter.
pub struct ConsoleTimingLoggerTextWriter<L: TimingLogger> {
    logger: L,
}

impl<L: TimingLogger> ConsoleTimingLoggerTextWriter<L> {
    pub fn new(logger: L) -> Self {
        Self { logger }
    }
}

impl<L: TimingLogger> Write for ConsoleTimingLoggerTextWriter<L> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s =
            std::str::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        self.logger.write_str(s);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<L: TimingLogger> ConsoleTimingLoggerTextWriter<L> {
    pub fn write_char_value(&mut self, value: char) {
        self.logger.write_char(value);
    }

    pub fn write_string_value(&mut self, value: &str) {
        self.logger.write_str(value);
    }

    pub fn write_line_empty(&mut self) {
        self.logger.write_line("");
    }

    pub fn write_line_char(&mut self, value: char) {
        let mut buf = [0u8; 4];
        let s = value.encode_utf8(&mut buf);
        self.logger.write_line(s);
    }

    pub fn write_line_string(&mut self, value: &str) {
        self.logger.write_line(value);
    }
}
