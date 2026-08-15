// Ported from Loretta.CodeAnalysis.Lua.CommandLine (b767b4e): Program
// C# source: src/Compilers/Lua/CommandLine/Program.cs

pub mod console_timing_logger_text_writer;

use console_timing_logger_text_writer::{ConsoleTimingLoggerTextWriter, TimingLogger};
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

/// The console logger. C# `ConsoleTimingLogger` is an external type
/// (Tsu.Timing) — dropped per the Port Boundary; the port carries a minimal
/// console logger with the same observable surface (timing prefixes are
/// runtime-dependent and not byte-comparable, so the oracle compares the
/// command outputs, not the prefixes).
pub struct ConsoleTimingLogger;

impl ConsoleTimingLogger {
    /// Writes a line to the console (C# ConsoleTimingLogger.WriteLine).
    pub fn write_line(&self, s: &str) {
        println!("{s}");
    }

    /// Writes to the console without a trailing newline (C# ConsoleTimingLogger.Write).
    pub fn write(&self, s: &str) {
        print!("{s}");
        io::stdout().flush().expect("flush stdout");
    }
}

impl TimingLogger for ConsoleTimingLogger {
    fn write_str(&self, s: &str) {
        ConsoleTimingLogger::write(self, s);
    }

    fn write_char(&self, c: char) {
        let mut buf = [0u8; 4];
        ConsoleTimingLogger::write(self, c.encode_utf8(&mut buf));
    }

    fn write_line(&self, s: &str) {
        ConsoleTimingLogger::write_line(self, s);
    }
}

impl TimingLogger for &ConsoleTimingLogger {
    fn write_str(&self, s: &str) {
        ConsoleTimingLogger::write(self, s);
    }

    fn write_char(&self, c: char) {
        let mut buf = [0u8; 4];
        ConsoleTimingLogger::write(self, c.encode_utf8(&mut buf));
    }

    fn write_line(&self, s: &str) {
        ConsoleTimingLogger::write_line(self, s);
    }
}

/// The REPL's logger (C# Program.s_logger).
static S_LOGGER: ConsoleTimingLogger = ConsoleTimingLogger;

/// Whether the REPL should keep running (C# Program.s_shouldRun).
static S_SHOULD_RUN: AtomicBool = AtomicBool::new(false);

/// Whether the REPL prints the current directory at each prompt (C# Program.s_printCurrentDir).
static S_PRINT_CURRENT_DIR: AtomicBool = AtomicBool::new(false);

/// Whether the REPL prefixes output with the timing logger (C# Program.s_printOutputPrefixed).
static S_PRINT_OUTPUT_PREFIXED: AtomicBool = AtomicBool::new(false);

/// A REPL command (C# System.CommandLine.Command — dropped infra; the port
/// carries the name/aliases/handler surface the REPL uses).
pub struct Command {
    /// The primary name (e.g. "q").
    pub name: &'static str,
    /// The aliases (e.g. "quit", "exit").
    pub aliases: &'static [&'static str],
    /// The handler invoked with the rest of the input line.
    pub handler: fn(&str),
}

/// The root command table (C# Program.s_rootCommand, built in the static ctor).
static S_ROOT_COMMAND: OnceLock<Vec<Command>> = OnceLock::new();

/// C# Program.Setting (private enum).
enum Setting {
    PrintCurrentDir,
    PrintOutputPrefixed,
}

/// C# Program.OutputWriter:
/// `s_printOutputPrefixed ? new ConsoleTimingLoggerTextWriter(s_logger) : Console.Out`.
fn output_writer() -> Box<dyn Write> {
    if S_PRINT_OUTPUT_PREFIXED.load(Ordering::Relaxed) {
        Box::new(ConsoleTimingLoggerTextWriter::new(&S_LOGGER))
    } else {
        Box::new(io::stdout())
    }
}

fn main() {
    // Temporary REPL stub mirroring Main's state initialization until the
    // Main row (410) lands (C# Main sets s_shouldRun = true first).
    S_SHOULD_RUN.store(true, Ordering::Relaxed);
    // C# Main: if (s_printCurrentDir) s_logger.Write(Environment.CurrentDirectory);
    if S_PRINT_CURRENT_DIR.load(Ordering::Relaxed) {
        if let Ok(dir) = std::env::current_dir() {
            S_LOGGER.write(&dir.display().to_string());
        }
    }
    // Referenced until the static ctor (row 456) builds it and Main (410) invokes it.
    let _ = S_ROOT_COMMAND.get();
    // Referenced until Set (row 414) uses it.
    let _ = Setting::PrintCurrentDir;
    let _ = Setting::PrintOutputPrefixed;
    writeln!(
        output_writer(),
        "loretta-cli: pending port — see loretta-rs/PROGRESS.md"
    )
    .expect("write output");
}
