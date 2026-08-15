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

    /// Logs an error to stderr (C# ConsoleTimingLogger.LogError).
    pub fn log_error(&self, s: &str) {
        eprintln!("{s}");
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

/// C# Program.LuaSyntaxOptionsPreset (private enum, Program.cs:124-137).
enum LuaSyntaxOptionsPreset {
    Lua51,
    Lua52,
    Lua53,
    Lua54,
}

/// C# Program.ListSymbols — lists the current directory's entries
/// (Program.cs:112-118). The .NET enumeration order is unspecified, so the
/// port sorts each group for a deterministic oracle.
fn list_symbols() {
    if let Ok(dir) = std::env::current_dir() {
        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut dirs: Vec<String> = Vec::new();
            let mut files: Vec<String> = Vec::new();
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                if is_dir {
                    dirs.push(name);
                } else {
                    files.push(name);
                }
            }
            dirs.sort();
            files.sort();
            for d in dirs {
                S_LOGGER.write_line(&format!("./{d}/"));
            }
            for f in files {
                S_LOGGER.write_line(&format!("./{f}"));
            }
        }
    }
}

/// C# Program.Quit — stops the REPL loop.
fn quit() {
    S_SHOULD_RUN.store(false, Ordering::Relaxed);
}

/// C# Program.ChangeDirectory — changes the current directory (Program.cs:99-110).
fn change_directory(relative_path: &str) {
    let result =
        std::env::current_dir().and_then(|dir| std::env::set_current_dir(dir.join(relative_path)));
    if let Err(e) = result {
        S_LOGGER.log_error(&format!("Error while changing directory: {e}"));
    }
}

/// C# Program.Set(Setting, string) — applies a REPL setting.
fn set_setting(setting: Setting, value: &str) -> Result<(), String> {
    match setting {
        Setting::PrintCurrentDir => {
            S_PRINT_CURRENT_DIR.store(parse_bool(value)?, Ordering::Relaxed);
        }
        Setting::PrintOutputPrefixed => {
            S_PRINT_OUTPUT_PREFIXED.store(parse_bool(value)?, Ordering::Relaxed);
        }
    }
    Ok(())
}

/// C# Program.Set's local ParseBool (Program.cs:81-90).
fn parse_bool(input: &str) -> Result<bool, String> {
    match input.to_lowercase().as_str() {
        "yes" | "true" | "on" => Ok(true),
        "no" | "false" | "off" => Ok(false),
        _ => Err(
            "Invalid boolean value '{0}' accepted values are: yes, true, on, no, false or off"
                .to_string(),
        ),
    }
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
    // Referenced until the static ctor (row 456) wires it into the command table.
    let _ = set_setting as fn(Setting, &str) -> Result<(), String>;
    let _ = quit as fn();
    let _ = change_directory as fn(&str);
    let _ = list_symbols as fn();
    // Constructed until the static ctor (row 456) wires the set command.
    let _ = (Setting::PrintCurrentDir, Setting::PrintOutputPrefixed);
    // Constructed until PresetEnumToPresetOptions (row 430) uses it.
    let _ = (
        LuaSyntaxOptionsPreset::Lua51,
        LuaSyntaxOptionsPreset::Lua52,
        LuaSyntaxOptionsPreset::Lua53,
        LuaSyntaxOptionsPreset::Lua54,
    );
    writeln!(
        output_writer(),
        "loretta-cli: pending port — see loretta-rs/PROGRESS.md"
    )
    .expect("write output");
}
