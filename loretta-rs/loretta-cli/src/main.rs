// Ported from Loretta.CLI.Program (b767b4e): Program
// C# source: src/Compilers/Lua/CommandLine/Program.cs

pub mod console_timing_logger_text_writer;

use console_timing_logger_text_writer::{ConsoleTimingLoggerTextWriter, TimingLogger};
use full_moon::tokenizer::{Symbol, Token, TokenReference, TokenType};
use full_moon::visitors::VisitorMut;
use loretta::experimental::minifying::islotallocator::ISlotAllocator;
use loretta::experimental::minifying::namingstrategy::NamingStrategy;
use loretta::experimental::minifying::sequentialslotallocator::SequentialSlotAllocator;
use loretta::experimental::minifying::sortedslotallocator::SortedSlotAllocator;
use loretta::luasyntaxoptions::LuaSyntaxOptions;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Abstraction of the external Tsu.Timing.ConsoleTimingLogger.
/// In C# the logger writes console output prefixed with the current time in
/// the form `[HH:MM:SS.ffffff]` (see the `prefixTemplate` in `RunMultiLua`).
/// The port implements that behavior with std::time::Instant.
#[derive(Clone)]
pub struct ConsoleTimingLogger {
    start: Instant,
}

impl ConsoleTimingLogger {
    /// Creates a new logger.
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// The timing prefix for the current moment.
    fn prefix(&self) -> String {
        let elapsed = self.start.elapsed();
        format!("[{}] ", format_duration(elapsed))
    }

    /// Writes the provided text without a prefix or newline.
    pub fn write(&self, value: &str) {
        let mut stdout = io::stdout().lock();
        let _ = stdout.write_all(value.as_bytes());
        let _ = stdout.flush();
    }

    /// Writes the provided text as a prefixed line.
    pub fn write_line(&self, value: &str) {
        let mut stdout = io::stdout().lock();
        let _ = writeln!(stdout, "{}{}", self.prefix(), value);
        let _ = stdout.flush();
    }

    /// Reads a line from standard input.
    pub fn read_line(&self) -> Option<String> {
        let mut line = String::new();
        match io::stdin().lock().read_line(&mut line) {
            Ok(0) => None,
            Ok(_) => Some(line.trim_end_matches(['\r', '\n']).to_string()),
            Err(_) => None,
        }
    }

    /// Logs an error message (prefixed, to standard error).
    pub fn log_error(&self, value: &str) {
        let mut stderr = io::stderr().lock();
        let _ = writeln!(stderr, "{}ERROR: {}", self.prefix(), value);
        let _ = stderr.flush();
    }

    /// Logs an information message (prefixed line).
    pub fn log_information(&self, value: &str) {
        self.write_line(value);
    }

    /// Begins a timed operation; the duration is logged when the returned
    /// guard is dropped (mirrors C# `using (BeginOperation(...))`).
    pub fn begin_operation(&self, name: &str) -> TimingOperation<'_> {
        TimingOperation {
            logger: self,
            name: name.to_string(),
            start: Instant::now(),
        }
    }
}

impl TimingLogger for ConsoleTimingLogger {
    fn write_str(&self, s: &str) {
        self.write(s);
    }

    fn write_char(&self, c: char) {
        self.write(&c.to_string());
    }

    fn write_line(&self, s: &str) {
        ConsoleTimingLogger::write_line(self, s);
    }
}

impl Default for ConsoleTimingLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// The scope guard returned by [`ConsoleTimingLogger::begin_operation`].
pub struct TimingOperation<'a> {
    logger: &'a ConsoleTimingLogger,
    name: String,
    start: Instant,
}

impl Drop for TimingOperation<'_> {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        self.logger
            .write_line(&format!("{} took {}", self.name, format_duration(elapsed)));
    }
}

/// Formats a duration as `HH:MM:SS.ffffff` (Tsu.Timing `Duration.Format` adaptation).
fn format_duration(duration: Duration) -> String {
    let total_micros = duration.as_micros();
    let hours = total_micros / 3_600_000_000;
    let minutes = (total_micros / 60_000_000) % 60;
    let seconds = (total_micros / 1_000_000) % 60;
    let micros = total_micros % 1_000_000;
    format!("{hours:02}:{minutes:02}:{seconds:02}.{micros:06}")
}

/// Formats a byte count (Tsu.Numerics `FileSize.Format` adaptation).
fn format_file_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.2} {}", UNITS[unit])
}

/// The process's private memory usage in bytes.
/// C# `Process.PrivateMemorySize64`; no std equivalent, so it is read from
/// `/proc/self/statm` on Linux or `ps -o rss=` on other unix systems.
/// Returns 0 if unavailable.
fn process_private_memory_bytes() -> u64 {
    // The C# `s_currentProc` field backs this read on every platform.
    let _pid = Program::current_process_id();
    #[cfg(target_os = "linux")]
    {
        if let Ok(statm) = std::fs::read_to_string("/proc/self/statm") {
            if let Some(pages) = statm.split_whitespace().nth(1) {
                if let Ok(pages) = pages.parse::<u64>() {
                    return pages * 4096;
                }
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        if let Ok(output) = Command::new("ps")
            .args(["-o", "rss=", "-p"])
            .arg(_pid.to_string())
            .output()
        {
            if let Ok(kib) = String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse::<u64>()
            {
                return kib * 1024;
            }
        }
    }
    0
}

/// The CLI program (C# `internal static class Program`).
pub struct Program;

/// The crate entry point; `Program::main` is the C# `Program.Main()` REPL.
fn main() {
    Program::main();
}

impl Program {
    /// C# `s_logger` field.
    fn logger() -> &'static ConsoleTimingLogger {
        static S_LOGGER: OnceLock<ConsoleTimingLogger> = OnceLock::new();
        S_LOGGER.get_or_init(ConsoleTimingLogger::new)
    }

    /// C# `s_shouldRun` field.
    fn should_run() -> &'static AtomicBool {
        static S_SHOULD_RUN: AtomicBool = AtomicBool::new(true);
        &S_SHOULD_RUN
    }

    /// C# `s_printCurrentDir` field.
    fn print_current_dir() -> &'static AtomicBool {
        static S_PRINT_CURRENT_DIR: AtomicBool = AtomicBool::new(false);
        &S_PRINT_CURRENT_DIR
    }

    /// C# `s_printOutputPrefixed` field.
    fn print_output_prefixed() -> &'static AtomicBool {
        static S_PRINT_OUTPUT_PREFIXED: AtomicBool = AtomicBool::new(false);
        &S_PRINT_OUTPUT_PREFIXED
    }

    /// C# `s_rootCommand` field.
    fn root_command() -> &'static Vec<CliCommand> {
        static S_ROOT_COMMAND: OnceLock<Vec<CliCommand>> = OnceLock::new();
        S_ROOT_COMMAND.get_or_init(build_root_command)
    }

    /// C# `s_memoryStack` field.
    fn memory_stack() -> &'static Mutex<Vec<(u64, u64)>> {
        static S_MEMORY_STACK: Mutex<Vec<(u64, u64)>> = Mutex::new(Vec::new());
        &S_MEMORY_STACK
    }

    /// C# `s_currentProc` field (`Process.GetCurrentProcess()`).
    fn current_process_id() -> u32 {
        static S_CURRENT_PROCESS_ID: OnceLock<u32> = OnceLock::new();
        *S_CURRENT_PROCESS_ID.get_or_init(std::process::id)
    }

    /// C# `OutputWriter` property: the timing-prefixed writer when
    /// `s_printOutputPrefixed` is set, otherwise standard output.
    fn output_writer() -> Box<dyn Write> {
        if Self::print_output_prefixed().load(Ordering::Relaxed) {
            Box::new(ConsoleTimingLoggerTextWriter::new(Self::logger().clone()))
        } else {
            Box::new(io::stdout())
        }
    }

    /// C# `Main()`: the REPL loop.
    pub fn main() {
        Self::should_run().store(true, Ordering::Relaxed);

        while Self::should_run().load(Ordering::Relaxed) {
            if Self::print_current_dir().load(Ordering::Relaxed) {
                let current_dir = std::env::current_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                Self::logger().write(&current_dir);
            }
            Self::logger().write("> ");

            let Some(line) = Self::logger().read_line() else {
                // C# `ReadLine() ?? throw ...` spins forever on EOF; the port
                // quits cleanly instead (documented adaptation).
                break;
            };

            let space_idx = line.find(' ');
            if let Some(space_idx) = space_idx {
                let verb = &line[..space_idx];
                let rest = &line[space_idx + 1..];
                if matches!(verb, "e" | "expr" | "expression") && !matches!(rest, "-h" | "--help") {
                    Self::parse_expression(rest);
                    continue;
                } else if matches!(verb, "emlua" | "expr-multi-lua" | "exprmultilua")
                    && !matches!(rest, "-h" | "--help")
                {
                    Self::multi_lua_expression(rest);
                    continue;
                }
            }
            // C# `var timingConsole = new TimingLoggerConsole(s_logger);`
            // created in `Main()` and passed to `s_rootCommand.Invoke(line, timingConsole)`.
            let timing_console = TimingLoggerConsole::new(Self::logger());
            Self::invoke(&line, &timing_console);
        }
    }

    /// Invokes the root command with the provided line (C# `s_rootCommand.Invoke(...)`).
    fn invoke(line: &str, timing_console: &TimingLoggerConsole) {
        let mut words = line.split_whitespace();
        let Some(name) = words.next() else {
            return;
        };
        let rest = words.collect::<Vec<_>>().join(" ");
        if let Some(command) = Self::root_command().iter().find(|c| c.matches(name)) {
            // C# `-h`/`--help` shows the command's help description.
            if matches!(rest.as_str(), "-h" | "--help") {
                timing_console.out().write(command.description);
                return;
            }
            (command.handler)(&rest);
        } else {
            timing_console
                .error()
                .write(&format!("Unrecognized command or argument '{name}'."));
        }
    }

    /// C# `Set(Setting, string)`.
    fn set_setting(args: &str) {
        let mut words = args.split_whitespace();
        let (Some(setting_name), Some(value)) = (words.next(), words.next()) else {
            Self::logger().write_line("Usage: s <setting> <value>");
            return;
        };
        let Some(setting) = Setting::parse(setting_name) else {
            Self::logger().log_error(&format!(
                "Unrecognized setting '{setting_name}'. Accepted values are: PrintCurrentDir, PrintOutputPrefixed"
            ));
            return;
        };
        match setting {
            Setting::PrintCurrentDir => {
                Self::print_current_dir().store(parse_bool(value), Ordering::Relaxed)
            }
            Setting::PrintOutputPrefixed => {
                Self::print_output_prefixed().store(parse_bool(value), Ordering::Relaxed)
            }
        }
    }

    /// C# `Quit()`.
    fn quit() {
        Self::should_run().store(false, Ordering::Relaxed);
    }

    /// C# `ChangeDirectory(string)`.
    fn change_directory(relative_path: &str) {
        let current_dir = std::env::current_dir().unwrap_or_default();
        let target = current_dir.join(relative_path);
        if let Err(err) = std::env::set_current_dir(&target) {
            Self::logger().log_error(&format!("Error while changing directory: {err}"));
        }
    }

    /// C# `ListSymbols()`.
    fn list_symbols() {
        let Ok(entries) = std::fs::read_dir(std::env::current_dir().unwrap_or_default()) else {
            return;
        };
        let entries: Vec<_> = entries.flatten().collect();
        for entry in entries.iter().filter(|e| e.path().is_dir()) {
            Self::logger().write_line(&format!("./{}/", entry.file_name().to_string_lossy()));
        }
        for entry in entries.iter().filter(|e| e.path().is_file()) {
            Self::logger().write_line(&format!("./{}", entry.file_name().to_string_lossy()));
        }
    }

    /// C# `PresetEnumToPresetOptions(LuaSyntaxOptionsPreset)`.
    fn preset_enum_to_preset_options(
        preset: LuaSyntaxOptionsPreset,
    ) -> loretta::luaparseoptions::LuaParseOptions {
        let syntax_options = match preset {
            LuaSyntaxOptionsPreset::Lua51 => LuaSyntaxOptions::LUA51,
            LuaSyntaxOptionsPreset::Lua52 => LuaSyntaxOptions::LUA52,
            LuaSyntaxOptionsPreset::Lua53 => LuaSyntaxOptions::LUA53,
            LuaSyntaxOptionsPreset::Lua54 => LuaSyntaxOptions::LUA54,
            LuaSyntaxOptionsPreset::LuaJit20 => LuaSyntaxOptions::LUAJIT20,
            LuaSyntaxOptionsPreset::LuaJit21 => LuaSyntaxOptions::LUAJIT21,
            LuaSyntaxOptionsPreset::GMod => LuaSyntaxOptions::GMOD,
            LuaSyntaxOptionsPreset::Luau => LuaSyntaxOptions::LUAU,
            LuaSyntaxOptionsPreset::FiveM => LuaSyntaxOptions::FIVEM,
            LuaSyntaxOptionsPreset::All => LuaSyntaxOptions::ALL,
            LuaSyntaxOptionsPreset::Alli => LuaSyntaxOptions::ALL_WITH_INTEGERS,
        };
        loretta::luaparseoptions::LuaParseOptions::new(syntax_options)
    }

    /// Reads a file and returns its contents, logging the C# error if missing.
    fn read_file_or_log(path: &str) -> Option<String> {
        if !Path::new(path).exists() {
            Self::logger().log_error("Provided path does not exist.");
            return None;
        }
        match std::fs::read_to_string(path) {
            Ok(contents) => Some(contents),
            Err(err) => {
                Self::logger().log_error(&format!("Error while reading file: {err}"));
                None
            }
        }
    }

    /// C# `Lex(LuaSyntaxOptionsPreset, string, bool)`.
    /// The C# token dump goes through `LuaTreeDumperConverter`/`TreeDumper`
    /// (dropped infra); the port prints one token per line instead.
    fn lex(args: &str) {
        let (preset, path, print_tokens) = parse_lex_args(args);
        let _ = preset;
        let Some(path) = path else {
            Self::logger().write_line("Usage: l <path> [-p <preset>] [--print-tokens]");
            return;
        };
        let Some(contents) = Self::read_file_or_log(&path) else {
            return;
        };

        let result = full_moon::parse_fallible(&contents, full_moon::LuaVersion::new());
        let ast = result.ast().clone();
        let mut collector = TokenCollector::new();
        collector.visit_ast(ast.clone());
        let mut tokens = collector.tokens;
        tokens.push(ast.eof().clone());

        Self::logger().log_information(&format!("{} tokens lexed.", tokens.len()));
        if !print_tokens {
            return;
        }
        let mut writer = Self::output_writer();
        for token_ref in &tokens {
            let kind = token_kind_name(token_ref.token());
            let _ = writeln!(writer, "{kind} {}", token_ref.token());
        }
    }

    /// C# `Parse(LuaSyntaxOptionsPreset, string, bool, bool, bool)`.
    fn parse(args: &str) {
        let (preset, path, constant_fold, print_tree, assume_no_overrides) = parse_parse_args(args);
        let _ = preset;
        let _ = assume_no_overrides;
        let Some(path) = path else {
            Self::logger().write_line("Usage: p <path> [-p <preset>] [-c] [-t] [-a]");
            return;
        };
        let Some(contents) = Self::read_file_or_log(&path) else {
            return;
        };

        let (root_node, errors) = {
            let _operation = Self::logger().begin_operation("Parsing");
            let result = full_moon::parse_fallible(&contents, full_moon::LuaVersion::new());
            let errors = result.errors().to_vec();
            (result.into_ast(), errors)
        };

        if constant_fold {
            // C# `rootNode.ConstantFold(new(assumeNoOverrides))` — the
            // ConstantFolder port is pending (PROGRESS rows 471-493).
            let _operation = Self::logger().begin_operation("Constant Folding");
        }

        let mut writer = Self::output_writer();
        {
            // C# `rootNode.NormalizeWhitespace()` — full_moon has no formatter;
            // the lossless `Ast::to_string()` round-trip is the documented
            // adaptation (preserves the original whitespace).
            let _operation = Self::logger().begin_operation("Format");
            let text = root_node.to_string();
            let _ = writer.write_all(text.as_bytes());
        }

        // C# `syntaxTree.GetDiagnostics()` — parse-diagnostics mapping is
        // pending (see differential/src/ops.rs); full_moon parse errors are
        // printed as a documented adaptation.
        for error in &errors {
            Self::logger().write_line(&error.to_string());
        }
        Self::logger().write("Press any key to continue...");
        let _ = io::stdin().lock().read_line(&mut String::new());
        Self::logger().write_line("");

        if print_tree {
            // C# `TreeDumper.DumpCompact(LuaTreeDumperConverter.Convert(rootNode))`
            // — TreeDumper is dropped infra; the code text is printed instead.
            let _ = writer.write_all(root_node.to_string().as_bytes());
        } else {
            let _ = writer.write_all(root_node.to_string().as_bytes());
            let _ = writer.write_all(b"\n");
        }
        let _ = writer.flush();

        // C# `new Script([syntaxTree])` + `script.RootScope.DeclaredVariables`
        // — the Script/scoping port is pending (SCC cluster).
        Self::logger().write_line("Global variables:");
    }

    /// C# `ParseExpression(string)`.
    fn parse_expression(input: &str) {
        let mut preset = LuaSyntaxOptionsPreset::All;
        let mut code = input;
        if let Some(space_idx) = code.find(' ') {
            let preset_name = &code[..space_idx];
            if let Some(parsed) = LuaSyntaxOptionsPreset::parse(preset_name) {
                code = &code[space_idx + 1..];
                preset = parsed;
            }
        }
        let _options = Self::preset_enum_to_preset_options(preset);

        let result = full_moon::parse_fallible(code, full_moon::LuaVersion::new());
        let errors = result.errors().to_vec();
        let expr = result.into_ast();
        for error in &errors {
            Self::logger().write_line(&error.to_string());
        }

        // C# `expr.ConstantFold(ConstantFoldingOptions.All)` — pending (rows 471-493).
        // C# `expr.NormalizeWhitespace()` — full_moon has no formatter; lossless
        // `to_string()` is the documented adaptation.
        let mut writer = Self::output_writer();
        let _ = writer.write_all(expr.to_string().as_bytes());
        let _ = writer.write_all(b"\n");
        let _ = writer.flush();
    }

    /// C# `MassParse(LuaSyntaxOptionsPreset, params string[])`.
    fn mass_parse(args: &str) {
        let (preset, patterns) = parse_preset_and_positionals(args);
        let _ = preset;
        let mut files = Vec::new();
        let Ok(entries) = std::fs::read_dir(".") else {
            return;
        };
        for entry in entries.flatten() {
            if !entry.path().is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if patterns.iter().any(|pattern| simple_match(pattern, &name)) {
                files.push(name);
            }
        }

        for file in files {
            let Some(contents) = Self::read_file_or_log(&file) else {
                continue;
            };
            let start = Instant::now();
            let result = full_moon::parse_fallible(&contents, full_moon::LuaVersion::new());
            let elapsed = start.elapsed();
            let has_errors = !result.errors().is_empty();
            Self::logger().write_line(&format!("{file}: {}", format_duration(elapsed)));
            // C# `if (!tree.GetRoot().ContainsDiagnostics) LogError(...)` —
            // ported verbatim (the C# condition is inverted).
            if !has_errors {
                Self::logger().log_error("Diagnostics were emitted.");
            }
        }
    }

    /// C# `GetNamingStrategy(NamingStrategy)`.
    /// `Minifying.NamingStrategies` is pending (PROGRESS rows 532-538); the
    /// placeholder is replaced when it lands.
    fn get_naming_strategy(naming_strategy: NamingStrategyEnum) -> NamingStrategy {
        let _ = naming_strategy;
        // Placeholder: maps a slot to its decimal representation until the
        // Alphabetical/Numerical/ZeroWidth strategies land.
        |slot| slot.to_string()
    }

    /// C# `GetSlotAllocator(SlotAllocator)`.
    fn get_slot_allocator(slot_allocator: SlotAllocatorEnum) -> Box<dyn ISlotAllocator> {
        match slot_allocator {
            SlotAllocatorEnum::Sequential => Box::new(SequentialSlotAllocator::new()),
            SlotAllocatorEnum::Sorted => Box::new(SortedSlotAllocator::new()),
        }
    }

    /// C# `Minify(string, LuaSyntaxOptionsPreset, NamingStrategy, SlotAllocator, bool)`.
    fn minify(args: &str) {
        let (path, preset, naming, allocator, format) = parse_minify_args(args);
        let _ = preset;
        let _ = format;
        let Some(path) = path else {
            Self::logger()
                .write_line("Usage: min <path> [-p <preset>] [-n <naming>] [-a <allocator>] [-f]");
            return;
        };
        let Some(contents) = Self::read_file_or_log(&path) else {
            return;
        };

        let ast = {
            let _operation = Self::logger().begin_operation("Parsing");
            full_moon::parse_fallible(&contents, full_moon::LuaVersion::new()).into_ast()
        };
        let _strategy = Self::get_naming_strategy(naming);
        let _allocator = Self::get_slot_allocator(allocator);
        {
            // C# `syntaxTree.Minify(...)` — the minifying port is pending
            // (PROGRESS rows 525+); the parsed code is printed as-is.
            let _operation = Self::logger().begin_operation("Minifying");
        }

        let mut writer = Self::output_writer();
        {
            let _operation = Self::logger().begin_operation("Formatting");
            let _ = writer.write_all(ast.to_string().as_bytes());
        }

        Self::logger().write("Press any key to continue...");
        let _ = io::stdin().lock().read_line(&mut String::new());
        Self::logger().write_line("");
        let _ = writer.write_all(b"\n");
        let _ = writer.flush();
    }

    /// C# `MultiLua(string)`.
    fn multi_lua(script_path: &str) {
        Self::run_multi_lua(std::slice::from_ref(&script_path.to_string()));
    }

    /// C# `MultiLuaExpression(string)`.
    fn multi_lua_expression(expression: &str) {
        let path = std::env::temp_dir().join("loretta-cli-multilua.lua");
        if std::fs::write(&path, expression).is_err() {
            return;
        }
        let path_string = path.display().to_string();
        Self::run_multi_lua(std::slice::from_ref(&path_string));
        let _ = std::fs::remove_file(path);
    }

    /// C# `RunMultiLua(params string[])`.
    fn run_multi_lua(args: &[String]) {
        const PREFIX_TEMPLATE: &str = "[00:00:00.000000]";
        // C# `Console.WindowWidth` has no std equivalent; 80 is the fallback
        // width used when the console is redirected.
        const CONSOLE_WIDTH: usize = 80;

        let mut versions = Vec::new();
        if let Ok(entries) = std::fs::read_dir("binaries") {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    versions.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
        versions.sort();

        for version in &versions {
            let name = version.replace('_', " ");
            let executable = format!("binaries/{version}/lua.exe");

            let title_prefix = format!("===== {name} ");
            let title = if title_prefix.len() >= CONSOLE_WIDTH.saturating_sub(PREFIX_TEMPLATE.len())
            {
                title_prefix
            } else {
                format!(
                    "{title_prefix}{}",
                    "=".repeat(
                        CONSOLE_WIDTH.saturating_sub(PREFIX_TEMPLATE.len()) - title_prefix.len()
                    )
                )
            };
            Self::logger().write_line(&title);

            let mut command = Command::new(&executable);
            command
                .args(args)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());
            let Ok(mut child) = command.spawn() else {
                Self::logger().log_error(&format!("Failed to start {executable}."));
                continue;
            };

            if let Some(stdout) = child.stdout.take() {
                std::thread::spawn(move || {
                    let reader = io::BufReader::new(stdout);
                    for line in reader.lines().map_while(Result::ok) {
                        Program::logger().write_line(&line);
                    }
                });
            }
            if let Some(stderr) = child.stderr.take() {
                std::thread::spawn(move || {
                    let reader = io::BufReader::new(stderr);
                    for line in reader.lines().map_while(Result::ok) {
                        Program::logger().log_error(&line);
                    }
                });
            }

            let deadline = Instant::now() + Duration::from_millis(2000);
            loop {
                match child.try_wait() {
                    Ok(Some(_)) => break,
                    Ok(None) if Instant::now() >= deadline => {
                        Self::logger().log_error("Process has timed out, killing...");
                        let _ = child.kill();
                        let _ = child.wait();
                        Self::logger().log_error("Killed.");
                        break;
                    }
                    Ok(None) => std::thread::sleep(Duration::from_millis(10)),
                    Err(_) => break,
                }
            }
        }
    }

    /// C# `Clear()`.
    fn clear() {
        print!("\x1b[2J\x1b[H");
        let _ = io::stdout().flush();
    }

    /// C# `PrintMemoryUsage()`.
    fn print_memory_usage() {
        // C# `GC.GetTotalMemory(false)` — Rust has no GC; 0 is reported.
        let gc_mem = 0u64;
        let proc_mem = process_private_memory_bytes();
        Self::logger().write_line(&format!(
            "Memory usage according to GC:       {}",
            format_file_size(gc_mem)
        ));
        Self::logger().write_line(&format!(
            "Memory usage according to Process:  {}",
            format_file_size(proc_mem)
        ));
    }

    /// C# `PushMemoryUsage()`.
    fn push_memory_usage() {
        let gc_mem = 0u64;
        let proc_mem = process_private_memory_bytes();
        Self::logger().write_line(&format!(
            "Memory usage according to GC:       {}",
            format_file_size(gc_mem)
        ));
        Self::logger().write_line(&format!(
            "Memory usage according to Process:  {}",
            format_file_size(proc_mem)
        ));
        if let Ok(mut stack) = Self::memory_stack().lock() {
            stack.push((gc_mem, proc_mem));
        }
        Self::logger().write_line("Memory usage pushed to stack.");
    }

    /// C# `CompareMemoryUsage()`.
    fn compare_memory_usage() {
        let curr_gc_mem = 0u64;
        let curr_proc_mem = process_private_memory_bytes();
        Self::logger().write_line(&format!(
            "Memory usage according to GC:       {}",
            format_file_size(curr_gc_mem)
        ));
        Self::logger().write_line(&format!(
            "Memory usage according to Process:  {}",
            format_file_size(curr_proc_mem)
        ));

        let Ok(stack) = Self::memory_stack().lock() else {
            return;
        };
        let Some(&(old_gc_mem, old_proc_mem)) = stack.last() else {
            Self::logger().log_error("Nothing on memory stack to compare to.");
            return;
        };

        let delta_gc = curr_gc_mem as i128 - old_gc_mem as i128;
        let delta_proc = curr_proc_mem as i128 - old_proc_mem as i128;
        Self::logger().write_line(&format!(
            "ΔMemory usage according to GC:      {}",
            format_signed_size(delta_gc)
        ));
        Self::logger().write_line(&format!(
            "ΔMemory usage according to Process: {}",
            format_signed_size(delta_proc)
        ));
    }

    /// C# `PopMemoryUsage()`.
    fn pop_memory_usage() {
        let is_empty = {
            let Ok(stack) = Self::memory_stack().lock() else {
                return;
            };
            stack.is_empty()
        };
        if is_empty {
            Self::logger().log_error("Nothing on memory stack to pop.");
            return;
        }

        Self::compare_memory_usage();
        if let Ok(mut stack) = Self::memory_stack().lock() {
            stack.pop();
        }
    }

    /// C# `InvokeGc(int)` — Rust has no GC; the loop is a documented no-op.
    fn invoke_gc(args: &str) {
        let amount = args
            .split_whitespace()
            .next()
            .and_then(|a| a.parse::<i32>().ok())
            .unwrap_or(1000);
        for _ in 0..amount {
            // C# `GC.Collect(...)` + `WaitForPendingFinalizers()` — no Rust equivalent.
        }
    }

    /// C# `m`/`mem` command with its push/pop/comp/compare subcommands.
    fn memory_command(args: &str) {
        let first = args.split_whitespace().next().unwrap_or("");
        match first {
            "push" => Self::push_memory_usage(),
            "pop" => Self::pop_memory_usage(),
            "comp" | "compare" => Self::compare_memory_usage(),
            _ => Self::print_memory_usage(),
        }
    }
}

/// Ported from Loretta.CLI.TimingLoggerConsole (b767b4e): TimingLoggerConsole, Writer
/// C# source: src/Compilers/Lua/CommandLine/TimingLoggerConsole.cs
///
/// System.CommandLine `IConsole` implementation backed by the timing logger.
/// The C# `Writer` uses reflection to call the private
/// `TimingLogger.ProcessWrite(LogLevel, string)`; the port calls the logger's
/// write/log_error directly (documented adaptation).
pub struct TimingLoggerConsole {
    out_writer: Writer,
    error_writer: Writer,
}

impl TimingLoggerConsole {
    /// C# `TimingLoggerConsole(TimingLogger)` ctor.
    pub fn new(logger: &ConsoleTimingLogger) -> Self {
        Self {
            out_writer: Writer::new(logger, LogLevel::None),
            error_writer: Writer::new(logger, LogLevel::Error),
        }
    }

    /// C# `Out` property.
    pub fn out(&self) -> &Writer {
        &self.out_writer
    }

    /// C# `IsOutputRedirected` property.
    pub fn is_output_redirected(&self) -> bool {
        false
    }

    /// C# `Error` property.
    pub fn error(&self) -> &Writer {
        &self.error_writer
    }

    /// C# `IsErrorRedirected` property.
    pub fn is_error_redirected(&self) -> bool {
        false
    }

    /// C# `IsInputRedirected` property.
    pub fn is_input_redirected(&self) -> bool {
        false
    }
}

/// Tsu.Timing `LogLevel` — only the values used by `Writer` are ported.
#[derive(Clone, Copy, PartialEq)]
enum LogLevel {
    None,
    Error,
}

/// C# nested `TimingLoggerConsole.Writer` class.
pub struct Writer {
    log_level: LogLevel,
    logger: ConsoleTimingLogger,
}

impl Writer {
    /// C# `Writer(TimingLogger, LogLevel)` ctor.
    fn new(logger: &ConsoleTimingLogger, log_level: LogLevel) -> Self {
        Self {
            log_level,
            logger: logger.clone(),
        }
    }

    /// C# `Write(string)` — dispatches through the log level.
    pub fn write(&self, value: &str) {
        match self.log_level {
            LogLevel::None => self.logger.write_line(value),
            LogLevel::Error => self.logger.log_error(value),
        }
    }
}

/// The C# `RootCommand` equivalent: a command with aliases and a handler.
/// System.CommandLine (external) is replaced by this table.
struct CliCommand {
    name: &'static str,
    aliases: &'static [&'static str],
    description: &'static str,
    handler: fn(&str),
}

impl CliCommand {
    fn matches(&self, name: &str) -> bool {
        name == self.name || self.aliases.contains(&name)
    }
}

/// Builds the command table (C# static constructor).
fn build_root_command() -> Vec<CliCommand> {
    vec![
        CliCommand {
            name: "@cd",
            aliases: &[],
            description: "Enable or disable printing the current directory.",
            // C# declares an Argument but no handler; invoking it prints usage.
            handler: |_| Program::logger().write_line("Usage: @cd <value> ('on' or 'off')"),
        },
        CliCommand {
            name: "s",
            aliases: &["set"],
            description: "Set a setting.",
            handler: Program::set_setting,
        },
        CliCommand {
            name: "q",
            aliases: &["quit", "exit"],
            description: "Quit the program.",
            handler: |_| Program::quit(),
        },
        CliCommand {
            name: "cd",
            aliases: &[],
            description: "Changes the current directory.",
            handler: |args| Program::change_directory(args),
        },
        CliCommand {
            name: "ls",
            aliases: &[],
            description: "List the current directory's symbols.",
            handler: |_| Program::list_symbols(),
        },
        CliCommand {
            name: "l",
            aliases: &["lex"],
            description: "Lexes the provided file.",
            handler: Program::lex,
        },
        CliCommand {
            name: "p",
            aliases: &["parse"],
            description: "Parses the provided file.",
            handler: Program::parse,
        },
        CliCommand {
            name: "e",
            aliases: &["expr", "expression"],
            description: "Parses the provided expression.",
            handler: |args| Program::parse_expression(args),
        },
        CliCommand {
            name: "mp",
            aliases: &["mass-parse"],
            description: "Parses files en masse by finding them with the provided patterns.",
            handler: Program::mass_parse,
        },
        CliCommand {
            name: "min",
            aliases: &["minify"],
            description: "Minifies the provided file.",
            handler: Program::minify,
        },
        CliCommand {
            name: "mlua",
            aliases: &["multi-lua", "multilua"],
            description: "Executes a file in multiple lua distributions.",
            handler: |args| Program::multi_lua(args),
        },
        CliCommand {
            name: "emlua",
            aliases: &["expr-multi-lua", "exprmultilua"],
            description: "Executes an expression in multiple lua distributions.",
            handler: |args| Program::multi_lua_expression(args),
        },
        CliCommand {
            name: "clear",
            aliases: &["cls"],
            description: "Clears the console screen.",
            handler: |_| Program::clear(),
        },
        CliCommand {
            name: "m",
            aliases: &["mem"],
            description: "Prints the current memory usage.",
            handler: Program::memory_command,
        },
        CliCommand {
            name: "gc",
            aliases: &[],
            description: "Aggressively invokes the GC.",
            handler: Program::invoke_gc,
        },
    ]
}

/// C# `Setting` enum.
#[derive(Clone, Copy, PartialEq)]
enum Setting {
    PrintCurrentDir,
    PrintOutputPrefixed,
}

impl Setting {
    fn parse(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "printcurrentdir" => Some(Self::PrintCurrentDir),
            "printoutputprefixed" => Some(Self::PrintOutputPrefixed),
            _ => None,
        }
    }
}

/// C# local `ParseBool(string)`.
fn parse_bool(input: &str) -> bool {
    match input.to_ascii_lowercase().as_str() {
        "yes" | "true" | "on" => true,
        "no" | "false" | "off" => false,
        _ => panic!(
            "Invalid boolean value '{}' accepted values are: yes, true, on, no, false or off",
            input
        ),
    }
}

/// C# `LuaSyntaxOptionsPreset` enum.
#[derive(Clone, Copy, PartialEq)]
enum LuaSyntaxOptionsPreset {
    Lua51,
    Lua52,
    Lua53,
    Lua54,
    LuaJit20,
    LuaJit21,
    GMod,
    Luau,
    FiveM,
    All,
    Alli,
}

impl LuaSyntaxOptionsPreset {
    fn parse(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "lua51" => Some(Self::Lua51),
            "lua52" => Some(Self::Lua52),
            "lua53" => Some(Self::Lua53),
            "lua54" => Some(Self::Lua54),
            "luajit20" => Some(Self::LuaJit20),
            "luajit21" => Some(Self::LuaJit21),
            "gmod" => Some(Self::GMod),
            "luau" => Some(Self::Luau),
            "fivem" => Some(Self::FiveM),
            "all" => Some(Self::All),
            "alli" => Some(Self::Alli),
            _ => None,
        }
    }
}

/// C# `NamingStrategy` enum.
#[derive(Clone, Copy, PartialEq)]
enum NamingStrategyEnum {
    Alphabetical,
    Numerical,
    ZeroWidth,
}

impl NamingStrategyEnum {
    fn parse(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "alphabetical" => Some(Self::Alphabetical),
            "numerical" => Some(Self::Numerical),
            "zerowidth" => Some(Self::ZeroWidth),
            _ => None,
        }
    }
}

/// C# `SlotAllocator` enum.
#[derive(Clone, Copy, PartialEq)]
enum SlotAllocatorEnum {
    Sequential,
    Sorted,
}

impl SlotAllocatorEnum {
    fn parse(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "sequential" => Some(Self::Sequential),
            "sorted" => Some(Self::Sorted),
            _ => None,
        }
    }
}

/// Parses `-p`/`--preset` and positional arguments.
fn parse_preset_and_positionals(args: &str) -> (LuaSyntaxOptionsPreset, Vec<String>) {
    let mut preset = LuaSyntaxOptionsPreset::All;
    let mut positionals = Vec::new();
    let mut words = args.split_whitespace().peekable();
    while let Some(word) = words.next() {
        match word {
            "-p" | "--preset" => {
                if let Some(value) = words.next() {
                    preset =
                        LuaSyntaxOptionsPreset::parse(value).unwrap_or(LuaSyntaxOptionsPreset::All);
                }
            }
            other => positionals.push(other.to_string()),
        }
    }
    (preset, positionals)
}

/// Parses the `lex` command arguments.
fn parse_lex_args(args: &str) -> (LuaSyntaxOptionsPreset, Option<String>, bool) {
    let mut preset = LuaSyntaxOptionsPreset::All;
    let mut path = None;
    let mut print_tokens = false;
    let mut words = args.split_whitespace().peekable();
    while let Some(word) = words.next() {
        match word {
            "-p" | "--preset" => {
                if let Some(value) = words.next() {
                    preset =
                        LuaSyntaxOptionsPreset::parse(value).unwrap_or(LuaSyntaxOptionsPreset::All);
                }
            }
            "--print-tokens" => print_tokens = true,
            other => path = Some(other.to_string()),
        }
    }
    (preset, path, print_tokens)
}

/// Parses the `parse` command arguments.
fn parse_parse_args(args: &str) -> (LuaSyntaxOptionsPreset, Option<String>, bool, bool, bool) {
    let mut preset = LuaSyntaxOptionsPreset::All;
    let mut path = None;
    let mut constant_fold = false;
    let mut print_tree = false;
    let mut assume_no_overrides = false;
    let mut words = args.split_whitespace().peekable();
    while let Some(word) = words.next() {
        match word {
            "-p" | "--preset" => {
                if let Some(value) = words.next() {
                    preset =
                        LuaSyntaxOptionsPreset::parse(value).unwrap_or(LuaSyntaxOptionsPreset::All);
                }
            }
            "-c" | "--constant-fold" => constant_fold = true,
            "-t" | "--print-tree" => print_tree = true,
            "-a" | "--assume-no-overrides" => assume_no_overrides = true,
            other => path = Some(other.to_string()),
        }
    }
    (preset, path, constant_fold, print_tree, assume_no_overrides)
}

/// Parses the `minify` command arguments.
fn parse_minify_args(
    args: &str,
) -> (
    Option<String>,
    LuaSyntaxOptionsPreset,
    NamingStrategyEnum,
    SlotAllocatorEnum,
    bool,
) {
    let mut path = None;
    let mut preset = LuaSyntaxOptionsPreset::All;
    let mut naming = NamingStrategyEnum::Numerical;
    let mut allocator = SlotAllocatorEnum::Sorted;
    let mut format = false;
    let mut words = args.split_whitespace().peekable();
    while let Some(word) = words.next() {
        match word {
            "-p" | "--preset" => {
                if let Some(value) = words.next() {
                    preset =
                        LuaSyntaxOptionsPreset::parse(value).unwrap_or(LuaSyntaxOptionsPreset::All);
                }
            }
            "-n" | "--naming" | "--naming-strategy" => {
                if let Some(value) = words.next() {
                    if let Some(parsed) = NamingStrategyEnum::parse(value) {
                        naming = parsed;
                    }
                }
            }
            "-a" | "--allocator" | "--slot-allocator" => {
                if let Some(value) = words.next() {
                    if let Some(parsed) = SlotAllocatorEnum::parse(value) {
                        allocator = parsed;
                    }
                }
            }
            "-f" | "--format" => format = true,
            other => path = Some(other.to_string()),
        }
    }
    (path, preset, naming, allocator, format)
}

/// A simple `*`/`?` wildcard matcher for `MassParse` (C# `MatchType.Simple`).
fn simple_match(pattern: &str, name: &str) -> bool {
    let pattern = pattern.as_bytes();
    let name = name.as_bytes();
    fn match_at(pattern: &[u8], name: &[u8]) -> bool {
        if pattern.is_empty() {
            return name.is_empty();
        }
        match pattern[0] {
            b'*' => {
                for skip in 0..=name.len() {
                    if match_at(&pattern[1..], &name[skip..]) {
                        return true;
                    }
                }
                false
            }
            b'?' => !name.is_empty() && match_at(&pattern[1..], &name[1..]),
            c => !name.is_empty() && name[0] == c && match_at(&pattern[1..], &name[1..]),
        }
    }
    match_at(pattern, name)
}

/// Collects the non-trivia tokens of an AST in source order (EOF last),
/// mirroring the C# `SyntaxFactory.ParseTokens` surface via full_moon.
struct TokenCollector {
    tokens: Vec<TokenReference>,
}

impl TokenCollector {
    fn new() -> Self {
        Self { tokens: Vec::new() }
    }
}

impl VisitorMut for TokenCollector {
    fn visit_token_reference(&mut self, token_ref: TokenReference) -> TokenReference {
        if !is_trivia(token_ref.token()) {
            self.tokens.push(token_ref.clone());
        }
        token_ref
    }
}

fn is_trivia(token: &Token) -> bool {
    matches!(
        token.token_type(),
        TokenType::Whitespace { .. }
            | TokenType::SingleLineComment { .. }
            | TokenType::MultiLineComment { .. }
            | TokenType::Shebang { .. }
    )
}

/// Maps a full_moon token to a compact kind name for the token dump
/// (C# `LuaTreeDumperConverter`/`TreeDumper` are dropped infra).
fn token_kind_name(token: &Token) -> String {
    match token.token_type() {
        TokenType::Eof => "EndOfFileToken".to_string(),
        TokenType::Identifier { .. } => "IdentifierToken".to_string(),
        TokenType::Number { .. } => "NumericLiteralToken".to_string(),
        TokenType::StringLiteral { .. } => "StringLiteralToken".to_string(),
        TokenType::Symbol { symbol } => match symbol {
            Symbol::And => "AndKeyword".to_string(),
            Symbol::Break => "BreakKeyword".to_string(),
            Symbol::Do => "DoKeyword".to_string(),
            Symbol::Else => "ElseKeyword".to_string(),
            Symbol::ElseIf => "ElseIfKeyword".to_string(),
            Symbol::End => "EndKeyword".to_string(),
            Symbol::False => "FalseKeyword".to_string(),
            Symbol::For => "ForKeyword".to_string(),
            Symbol::Function => "FunctionKeyword".to_string(),
            Symbol::If => "IfKeyword".to_string(),
            Symbol::In => "InKeyword".to_string(),
            Symbol::Local => "LocalKeyword".to_string(),
            Symbol::Nil => "NilKeyword".to_string(),
            Symbol::Not => "NotKeyword".to_string(),
            Symbol::Or => "OrKeyword".to_string(),
            Symbol::Repeat => "RepeatKeyword".to_string(),
            Symbol::Return => "ReturnKeyword".to_string(),
            Symbol::Then => "ThenKeyword".to_string(),
            Symbol::True => "TrueKeyword".to_string(),
            Symbol::Until => "UntilKeyword".to_string(),
            Symbol::While => "WhileKeyword".to_string(),
            Symbol::Plus => "PlusToken".to_string(),
            Symbol::Minus => "MinusToken".to_string(),
            Symbol::Star => "StarToken".to_string(),
            Symbol::Slash => "SlashToken".to_string(),
            Symbol::DoubleSlash => "DoubleSlashToken".to_string(),
            Symbol::Percent => "PercentToken".to_string(),
            Symbol::Caret => "CaretToken".to_string(),
            Symbol::Hash => "HashToken".to_string(),
            Symbol::Ampersand => "AmpersandToken".to_string(),
            Symbol::Tilde => "TildeToken".to_string(),
            Symbol::TildeEqual => "TildeEqualsToken".to_string(),
            Symbol::Pipe => "PipeToken".to_string(),
            Symbol::DoubleLessThan => "DoubleLessThanToken".to_string(),
            Symbol::DoubleGreaterThan => "DoubleGreaterThanToken".to_string(),
            Symbol::Equal => "EqualsToken".to_string(),
            Symbol::TwoEqual => "TwoEqualsToken".to_string(),
            Symbol::LessThanEqual => "LessThanEqualsToken".to_string(),
            Symbol::GreaterThanEqual => "GreaterThanEqualsToken".to_string(),
            Symbol::LessThan => "LessThanToken".to_string(),
            Symbol::GreaterThan => "GreaterThanToken".to_string(),
            Symbol::TwoDots => "TwoDotsToken".to_string(),
            Symbol::Ellipsis => "EllipsisToken".to_string(),
            Symbol::Colon => "ColonToken".to_string(),
            Symbol::Semicolon => "SemicolonToken".to_string(),
            Symbol::Comma => "CommaToken".to_string(),
            Symbol::Dot => "DotToken".to_string(),
            Symbol::LeftBrace => "LeftBraceToken".to_string(),
            Symbol::RightBrace => "RightBraceToken".to_string(),
            Symbol::LeftBracket => "LeftBracketToken".to_string(),
            Symbol::RightBracket => "RightBracketToken".to_string(),
            Symbol::LeftParen => "LeftParenToken".to_string(),
            Symbol::RightParen => "RightParenToken".to_string(),
            Symbol::TwoColons => "TwoColonsToken".to_string(),
            Symbol::QuestionMark => "QuestionMarkToken".to_string(),
            Symbol::AtSign => "AtSignToken".to_string(),
            Symbol::ThinArrow => "ThinArrowToken".to_string(),
            Symbol::PlusEqual => "PlusEqualsToken".to_string(),
            Symbol::MinusEqual => "MinusEqualsToken".to_string(),
            Symbol::StarEqual => "StarEqualsToken".to_string(),
            Symbol::SlashEqual => "SlashEqualsToken".to_string(),
            Symbol::DoubleSlashEqual => "DoubleSlashEqualsToken".to_string(),
            Symbol::PercentEqual => "PercentEqualsToken".to_string(),
            Symbol::CaretEqual => "CaretEqualsToken".to_string(),
            Symbol::TwoDotsEqual => "TwoDotsEqualsToken".to_string(),
            Symbol::AmpersandEqual => "AmpersandEqualsToken".to_string(),
            Symbol::PipeEqual => "PipeEqualsToken".to_string(),
            Symbol::DoubleLessThanEqual => "DoubleLessThanEqualsToken".to_string(),
            Symbol::DoubleGreaterThanEqual => "DoubleGreaterThanEqualsToken".to_string(),
            Symbol::QuestionMarkDot => "QuestionMarkDotToken".to_string(),
            _ => format!("UNMAPPED_{:?}", token.token_type()),
        },
        _ => format!("UNMAPPED_{:?}", token.token_type()),
    }
}

/// Formats a signed byte delta (C# inline `-FileSize.Format(-delta)`).
fn format_signed_size(delta: i128) -> String {
    if delta < 0 {
        format!("-{}", format_file_size(delta.unsigned_abs() as u64))
    } else {
        format_file_size(delta as u64)
    }
}
