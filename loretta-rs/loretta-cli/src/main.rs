// Ported from Loretta.CodeAnalysis.Lua.CommandLine (b767b4e): Program
// C# source: src/Compilers/Lua/CommandLine/Program.cs

pub mod console_timing_logger_text_writer;

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
}

/// The REPL's logger (C# Program.s_logger).
static S_LOGGER: ConsoleTimingLogger = ConsoleTimingLogger;

fn main() {
    S_LOGGER.write_line("loretta-cli: pending port — see loretta-rs/PROGRESS.md");
}
