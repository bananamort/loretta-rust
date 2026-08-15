// Ported from Loretta.CodeAnalysis.Lua.CommandLine (b767b4e): Program
// C# source: src/Compilers/Lua/CommandLine/Program.cs

pub mod console_timing_logger_text_writer;

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};

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

/// The REPL's logger (C# Program.s_logger).
static S_LOGGER: ConsoleTimingLogger = ConsoleTimingLogger;

/// Whether the REPL should keep running (C# Program.s_shouldRun).
static S_SHOULD_RUN: AtomicBool = AtomicBool::new(false);

/// Whether the REPL prints the current directory at each prompt (C# Program.s_printCurrentDir).
static S_PRINT_CURRENT_DIR: AtomicBool = AtomicBool::new(false);

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
    S_LOGGER.write_line("loretta-cli: pending port — see loretta-rs/PROGRESS.md");
}
