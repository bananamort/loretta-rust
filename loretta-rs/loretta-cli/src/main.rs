// Ported from Loretta.CodeAnalysis.Lua.CommandLine (b767b4e): Program
// C# source: src/Compilers/Lua/CommandLine/Program.cs

pub mod console_timing_logger_text_writer;

use console_timing_logger_text_writer::{ConsoleTimingLoggerTextWriter, TimingLogger};
use full_moon::tokenizer::Symbol;
use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

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

    /// Logs an informational line (C# ConsoleTimingLogger.LogInformation).
    pub fn log_information(&self, s: &str) {
        println!("{s}");
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

/// The current process (C# Program.s_currentProc = Process.GetCurrentProcess()).
fn current_proc() -> u32 {
    std::process::id()
}

/// The memory usage stack (C# Program.s_memoryStack, Stack<(gcMemory, processMemory)>).
static S_MEMORY_STACK: Mutex<Vec<(u64, u64)>> = Mutex::new(Vec::new());

/// The current process's resident memory in bytes (best-effort: /proc/self/statm
/// on Linux; 0 elsewhere — runtime data, not byte-comparable).
fn process_memory() -> u64 {
    if let Ok(statm) = std::fs::read_to_string("/proc/self/statm") {
        if let Some(rss) = statm.split_whitespace().nth(1) {
            if let Ok(pages) = rss.parse::<u64>() {
                return pages * 4096;
            }
        }
    }
    0
}

/// The GC-reported memory (C# GC.GetTotalMemory(false)) — the port reports the
/// process's resident memory (no GC in Rust; runtime data).
fn gc_memory() -> u64 {
    process_memory()
}

/// Renders a byte count like the dropped Tsu FileSize.Format.
fn file_size_format(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.2} {}", UNITS[unit])
}

/// C# Program.Setting (private enum, Program.cs:66-70). Applied by Set (row 414).
#[derive(Copy, Clone)]
enum Setting {
    PrintCurrentDir,
    PrintOutputPrefixed,
}

/// C# Program.SlotAllocator (private enum, Program.cs:302-306).
#[derive(Copy, Clone)]
enum SlotAllocator {
    Sequential,
    Sorted,
}

/// C# Program.NamingStrategy (private enum, Program.cs:284-289). Maps to
/// Minifying.NamingStrategies via GetNamingStrategy (row 439).
#[derive(Copy, Clone)]
enum NamingStrategy {
    Alphabetical,
    Numerical,
    ZeroWidth,
}

/// C# Program.LuaSyntaxOptionsPreset (private enum, Program.cs:124-137).
/// Maps to LuaSyntaxOptions via PresetEnumToPresetOptions (row 430).
#[derive(Copy, Clone)]
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

/// C# Program.ListSymbols — lists the current directory's entries
/// (Program.cs:112-118). The directory enumeration order is the OS order,
/// exactly as the C# EnumerateDirectories/EnumerateFiles (no sort).
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

/// C# Program.PresetEnumToPresetOptions (Program.cs:139-157).
fn preset_enum_to_preset_options(
    preset: LuaSyntaxOptionsPreset,
) -> loretta::luaparseoptions::LuaParseOptions {
    use loretta::luasyntaxoptions::LuaSyntaxOptions;
    loretta::luaparseoptions::LuaParseOptions::new(match preset {
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
    })
}

/// The full-moon LuaVersion for a preset (the CLI's lex/parse use full_moon
/// as the dropped lexer/parser replacement, per AGENTS.md).
fn preset_to_lua_version(preset: LuaSyntaxOptionsPreset) -> full_moon::LuaVersion {
    use full_moon::LuaVersion;
    match preset {
        LuaSyntaxOptionsPreset::Lua51 => LuaVersion::lua51(),
        LuaSyntaxOptionsPreset::Lua52 => LuaVersion::lua52(),
        LuaSyntaxOptionsPreset::Lua53 => LuaVersion::lua53(),
        LuaSyntaxOptionsPreset::Lua54 => LuaVersion::lua54(),
        LuaSyntaxOptionsPreset::LuaJit20 | LuaSyntaxOptionsPreset::LuaJit21 => LuaVersion::luajit(),
        LuaSyntaxOptionsPreset::Luau | LuaSyntaxOptionsPreset::GMod => LuaVersion::luau(),
        LuaSyntaxOptionsPreset::FiveM => LuaVersion::cfxlua(),
        LuaSyntaxOptionsPreset::All | LuaSyntaxOptionsPreset::Alli => LuaVersion::new(),
    }
}

/// Collects every TokenReference in the AST in source order, mirroring the
/// C# reference's token enumeration (trivia excluded; EOF included).
fn collect_tokens(ast: &full_moon::ast::Ast) -> Vec<full_moon::tokenizer::TokenReference> {
    use full_moon::visitors::VisitorMut;
    struct TokenCollector {
        tokens: Vec<full_moon::tokenizer::TokenReference>,
    }
    impl VisitorMut for TokenCollector {
        fn visit_token_reference(
            &mut self,
            token_ref: full_moon::tokenizer::TokenReference,
        ) -> full_moon::tokenizer::TokenReference {
            if !is_trivia(token_ref.token()) {
                self.tokens.push(token_ref.clone());
            }
            token_ref
        }
    }
    fn is_trivia(token: &full_moon::tokenizer::Token) -> bool {
        matches!(
            token.token_type(),
            full_moon::tokenizer::TokenType::Whitespace { .. }
                | full_moon::tokenizer::TokenType::SingleLineComment { .. }
                | full_moon::tokenizer::TokenType::MultiLineComment { .. }
                | full_moon::tokenizer::TokenType::Shebang { .. }
                | full_moon::tokenizer::TokenType::CStyleComment { .. }
        )
    }
    let mut visitor = TokenCollector { tokens: Vec::new() };
    visitor.visit_ast(ast.clone());
    let mut tokens = visitor.tokens;
    tokens.push(ast.eof().clone());
    tokens
}

/// Maps a full_moon token to the C# SyntaxKind name (oracle data from
/// Compilers/Lua/Portable/Syntax/SyntaxKind.cs + the corpus expected lex.json).
fn kind_name(token: &full_moon::tokenizer::Token) -> String {
    use full_moon::tokenizer::TokenType;
    match token.token_type() {
        TokenType::Eof => "EndOfFileToken".to_string(),
        TokenType::Identifier { .. } => "IdentifierToken".to_string(),
        TokenType::Number { .. } => "NumericLiteralToken".to_string(),
        TokenType::StringLiteral { .. } => "StringLiteralToken".to_string(),
        TokenType::Symbol { symbol } => symbol_kind_name(*symbol),
        _ => format!("UNMAPPED_{:?}", token.token_kind()),
    }
}

fn symbol_kind_name(symbol: Symbol) -> String {
    use full_moon::tokenizer::Symbol;
    let word = match symbol {
        Symbol::And => "And",
        Symbol::Break => "Break",
        Symbol::Do => "Do",
        Symbol::Else => "Else",
        Symbol::ElseIf => "ElseIf",
        Symbol::End => "End",
        Symbol::False => "False",
        Symbol::For => "For",
        Symbol::Function => "Function",
        Symbol::If => "If",
        Symbol::In => "In",
        Symbol::Local => "Local",
        Symbol::Nil => "Nil",
        Symbol::Not => "Not",
        Symbol::Or => "Or",
        Symbol::Repeat => "Repeat",
        Symbol::Return => "Return",
        Symbol::Then => "Then",
        Symbol::True => "True",
        Symbol::Until => "Until",
        Symbol::While => "While",
        Symbol::Goto => "Goto",
        _ => return symbol_operator_kind_name(symbol),
    };
    format!("{word}Keyword")
}

fn symbol_operator_kind_name(symbol: Symbol) -> String {
    use full_moon::tokenizer::Symbol;
    let name = match symbol {
        Symbol::PlusEqual => "PlusEquals",
        Symbol::MinusEqual => "MinusEquals",
        Symbol::StarEqual => "StarEquals",
        Symbol::SlashEqual => "SlashEquals",
        Symbol::DoubleSlashEqual => "SlashSlashEquals",
        Symbol::PercentEqual => "PercentEquals",
        Symbol::CaretEqual => "HatEquals",
        Symbol::TwoDotsEqual => "DotDotEquals",
        Symbol::Ampersand => "Ampersand",
        Symbol::ThinArrow => "MinusGreaterThan",
        Symbol::TwoColons => "ColonColon",
        Symbol::AtSign => "At",
        Symbol::DoubleLessThanEqual => "LessThanLessThanEquals",
        Symbol::DoubleGreaterThanEqual => "GreaterThanGreaterThanEquals",
        Symbol::AmpersandEqual => "AmpersandEquals",
        Symbol::PipeEqual => "PipeEquals",
        Symbol::QuestionMarkDot => "QuestionMarkDot",
        Symbol::Caret => "Hat",
        Symbol::Colon => "Colon",
        Symbol::Comma => "Comma",
        Symbol::Dot => "Dot",
        Symbol::TwoDots => "DotDot",
        Symbol::Ellipsis => "DotDotDot",
        Symbol::Equal => "Equals",
        Symbol::TwoEqual => "EqualsEquals",
        Symbol::GreaterThan => "GreaterThan",
        Symbol::GreaterThanEqual => "GreaterThanEquals",
        Symbol::DoubleGreaterThan => "GreaterThanGreaterThan",
        Symbol::Hash => "Hash",
        Symbol::LeftBrace => "OpenBrace",
        Symbol::LeftBracket => "OpenBracket",
        Symbol::LeftParen => "OpenParenthesis",
        Symbol::LessThan => "LessThan",
        Symbol::LessThanEqual => "LessThanEquals",
        Symbol::DoubleLessThan => "LessThanLessThan",
        Symbol::Minus => "Minus",
        Symbol::Percent => "Percent",
        Symbol::Pipe => "Pipe",
        Symbol::Plus => "Plus",
        Symbol::QuestionMark => "Question",
        Symbol::RightBrace => "CloseBrace",
        Symbol::RightBracket => "CloseBracket",
        Symbol::RightParen => "CloseParenthesis",
        Symbol::Semicolon => "Semicolon",
        Symbol::Slash => "Slash",
        Symbol::DoubleSlash => "SlashSlash",
        Symbol::Star => "Star",
        Symbol::Tilde => "Tilde",
        Symbol::TildeEqual => "TildeEquals",
        _ => return format!("UNMAPPED_SYMBOL_{symbol:?}"),
    };
    format!("{name}Token")
}

/// The token/trivia text rendered like the C# TreeDumper value (control
/// characters escaped as \n, \r, \t, \\, \0, \a, \b, \f, \v).
fn dump_value(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        match c {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\\' => out.push_str("\\\\"),
            '\0' => out.push_str("\\0"),
            '\x07' => out.push_str("\\a"),
            '\x08' => out.push_str("\\b"),
            '\x0C' => out.push_str("\\f"),
            '\x0B' => out.push_str("\\v"),
            c => out.push(c),
        }
    }
    out
}

/// The parsed value of a numeric literal rendered like the C# token
/// Value.ToString() (e.g. "0b1010" -> "10", "0x1A" -> "26", "1.5" -> "1.5").
fn numeric_value_text(text: &str) -> String {
    let cleaned: String = text.chars().filter(|c| *c != '_').collect();
    let is_float = cleaned.contains('.') || cleaned.contains('e') || cleaned.contains('E');
    if is_float {
        let value: f64 = parse_number(&cleaned);
        loretta::symbol_display::objectdisplay::ObjectDisplay::format_double_r(value)
    } else {
        parse_integer(&cleaned).to_string()
    }
}

fn parse_integer(text: &str) -> i64 {
    if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        i64::from_str_radix(hex, 16).expect("hex literal")
    } else if let Some(bin) = text.strip_prefix("0b").or_else(|| text.strip_prefix("0B")) {
        i64::from_str_radix(bin, 2).expect("binary literal")
    } else if let Some(oct) = text.strip_prefix("0o").or_else(|| text.strip_prefix("0O")) {
        i64::from_str_radix(oct, 8).expect("octal literal")
    } else {
        text.parse::<i64>().expect("decimal literal")
    }
}

fn parse_number(text: &str) -> f64 {
    if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        loretta::utilities::hexfloat::HexFloat::double_from_hex_string(hex)
            .expect("hex float literal")
    } else {
        text.parse::<f64>().expect("float literal")
    }
}

/// The decoded value of a string literal (quotes stripped, escapes decoded),
/// rendered like the C# token Value.ToString().
fn string_value_text(text: &str) -> String {
    if let Some(rest) = text.strip_prefix("[[") {
        let end = rest.find("]]").expect("long string end");
        return rest[..end].to_string();
    }
    if let Some(rest) = text.strip_prefix('"') {
        let end = rest.rfind('"').expect("string end quote");
        return decode_escapes(&rest[..end]);
    }
    if let Some(rest) = text.strip_prefix('\'') {
        let end = rest.rfind('\'').expect("string end quote");
        return decode_escapes(&rest[..end]);
    }
    if let Some(rest) = text.strip_prefix('`') {
        let end = rest.rfind('`').expect("backtick string end");
        return rest[..end].to_string();
    }
    text.to_string()
}

/// Decodes Lua string escapes (\n, \t, \\, \", \', \xXX, \u{...}, ...).
fn decode_escapes(input: &str) -> String {
    let mut out = String::new();
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some('0') => out.push('\0'),
            Some('a') => out.push('\x07'),
            Some('b') => out.push('\x08'),
            Some('f') => out.push('\x0C'),
            Some('v') => out.push('\x0B'),
            Some('x') => {
                let hex: String = chars.by_ref().take(2).collect();
                let value = u32::from_str_radix(&hex, 16).expect("hex escape");
                out.push(char::from_u32(value).expect("valid hex escape char"));
            }
            Some('u') => {
                let _ = chars.next(); // {
                let hex: String = chars.by_ref().take_while(|c| *c != '}').collect();
                let value = u32::from_str_radix(&hex, 16).expect("unicode escape");
                out.push(char::from_u32(value).expect("valid unicode escape char"));
            }
            Some(other) => out.push(other),
            None => {}
        }
    }
    out
}

/// Replicates the dropped TreeDumper.DumpCompact format (verified against the
/// reference package output): a tree of "name: value" lines with the
/// box-drawing ├─/│/└─ glyphs and a Diagnostics leaf per node.
fn dump_node(
    name: &str,
    value: Option<&str>,
    children: &[DumpChild],
    prefix: &str,
    is_last: bool,
) -> String {
    let mut out = String::new();
    let label = match value {
        Some(v) => format!("{name}: {v}"),
        None => name.to_string(),
    };
    out.push_str(prefix);
    out.push_str(if is_last { "└─" } else { "├─" });
    out.push_str(&label);
    out.push('\n');
    let child_prefix = format!("{prefix}{}", if is_last { "  " } else { "│ " });
    for (i, child) in children.iter().enumerate() {
        let last = i + 1 == children.len();
        out.push_str(&child.render(&child_prefix, last));
    }
    out
}

/// A node of the compact dump (the token, its trivia, or a Diagnostics leaf).
enum DumpChild {
    Node {
        name: String,
        value: Option<String>,
        children: Vec<DumpChild>,
    },
}

impl DumpChild {
    fn render(&self, prefix: &str, is_last: bool) -> String {
        match self {
            DumpChild::Node {
                name,
                value,
                children,
            } => dump_node(name, value.as_deref(), children, prefix, is_last),
        }
    }
}

/// Builds the dump for the lexed tokens: each token node has Leading Trivia,
/// Trailing Trivia and Diagnostics children (the reference TreeDumper shape).
fn build_token_dump(tokens: &[full_moon::tokenizer::TokenReference]) -> String {
    let children: Vec<DumpChild> = tokens
        .iter()
        .map(|t| {
            let trivia_kind_name = |trivia: &full_moon::tokenizer::Token| -> String {
                let text = trivia.to_string();
                match trivia.token_type() {
                    full_moon::tokenizer::TokenType::Whitespace { .. } => {
                        // The C# lexer splits whitespace trivia into
                        // WhitespaceTrivia and EndOfLineTrivia by content.
                        if text.contains('\n') || text.contains('\r') {
                            "EndOfLineTrivia".to_string()
                        } else {
                            "WhitespaceTrivia".to_string()
                        }
                    }
                    full_moon::tokenizer::TokenType::SingleLineComment { .. } => {
                        "SingleLineCommentTrivia".to_string()
                    }
                    full_moon::tokenizer::TokenType::MultiLineComment { .. } => {
                        "MultiLineCommentTrivia".to_string()
                    }
                    full_moon::tokenizer::TokenType::Shebang { .. } => "ShebangTrivia".to_string(),
                    full_moon::tokenizer::TokenType::CStyleComment { .. } => {
                        "CStyleCommentTrivia".to_string()
                    }
                    _ => format!("UNMAPPED_TRIVIA_{:?}", trivia.token_kind()),
                }
            };
            let trivia_nodes = |trivia: &[full_moon::tokenizer::Token]| -> Vec<DumpChild> {
                trivia
                    .iter()
                    .map(|tr| DumpChild::Node {
                        name: trivia_kind_name(tr),
                        value: Some(dump_value(&tr.to_string())),
                        children: vec![DumpChild::Node {
                            name: "Diagnostics".to_string(),
                            value: None,
                            children: Vec::new(),
                        }],
                    })
                    .collect()
            };
            let leading: Vec<full_moon::tokenizer::Token> = t.leading_trivia().cloned().collect();
            let trailing: Vec<full_moon::tokenizer::Token> = t.trailing_trivia().cloned().collect();
            let token_value = |t: &full_moon::tokenizer::TokenReference| -> String {
                let text = t.token().to_string();
                match t.token().token_type() {
                    full_moon::tokenizer::TokenType::Number { .. } => numeric_value_text(&text),
                    full_moon::tokenizer::TokenType::StringLiteral { .. } => {
                        string_value_text(&text)
                    }
                    _ => dump_value(&text),
                }
            };
            DumpChild::Node {
                name: kind_name(t.token()),
                value: Some(token_value(t)),
                children: vec![
                    DumpChild::Node {
                        name: "Leading Trivia".to_string(),
                        value: None,
                        children: trivia_nodes(&leading),
                    },
                    DumpChild::Node {
                        name: "Trailing Trivia".to_string(),
                        value: None,
                        children: trivia_nodes(&trailing),
                    },
                    DumpChild::Node {
                        name: "Diagnostics".to_string(),
                        value: None,
                        children: Vec::new(),
                    },
                ],
            }
        })
        .collect();
    // The root node has no branch prefix (reference TreeDumper.DumpCompact);
    // the dump itself has no trailing newline (the CLI writes it via WriteLine).
    let mut out = String::from("Root\n");
    for (i, child) in children.iter().enumerate() {
        let last = i + 1 == children.len();
        out.push_str(&child.render("", last));
    }
    out.pop();
    out
}

/// C# Program.Lex (Program.cs:159-180): lexes a file with the preset's
/// options (the dropped lexer replaced by full_moon) and optionally prints
/// the compact tree dump.
fn lex_command(preset: LuaSyntaxOptionsPreset, path: &str, print_tokens: bool) {
    if !std::path::Path::new(path).is_file() {
        S_LOGGER.log_error("Provided path does not exist.");
        return;
    }
    let code = std::fs::read_to_string(path).expect("read file");
    let ast = full_moon::parse_fallible(&code, preset_to_lua_version(preset))
        .into_result()
        .expect("parse");
    let tokens = collect_tokens(&ast);
    S_LOGGER.log_information(&format!("{} tokens lexed.", tokens.len()));
    if !print_tokens {
        return;
    }
    let dump = build_token_dump(&tokens);
    writeln!(output_writer(), "{dump}").expect("write output");
}

/// Simple wildcard match (the C# MatchType.Simple enumeration patterns).
fn glob_matches(name: &str, pattern: &str) -> bool {
    fn rec(n: &[char], p: &[char]) -> bool {
        if p.is_empty() {
            return n.is_empty();
        }
        match p[0] {
            '*' => {
                for i in 0..=n.len() {
                    if rec(&n[i..], &p[1..]) {
                        return true;
                    }
                }
                false
            }
            '?' => !n.is_empty() && rec(&n[1..], &p[1..]),
            c => !n.is_empty() && n[0] == c && rec(&n[1..], &p[1..]),
        }
    }
    let n: Vec<char> = name.chars().collect();
    let p: Vec<char> = pattern.chars().collect();
    rec(&n, &p)
}

/// C# Program.MassParse (Program.cs:262-282): parses files matching the
/// patterns. The enumeration order is the OS order, exactly as the C#
/// EnumerateFiles (no sort); the duration output is runtime data
/// (not byte-comparable).
fn mass_parse_command(preset: LuaSyntaxOptionsPreset, patterns: &[&str]) {
    let mut files: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(".") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if patterns.iter().any(|p| glob_matches(&name, p)) {
                files.push(name);
            }
        }
    }
    let _options = preset_enum_to_preset_options(preset);
    for file in files {
        match std::fs::read_to_string(&file) {
            Ok(code) => {
                let start = std::time::Instant::now();
                let result =
                    full_moon::parse_fallible(&code, preset_to_lua_version(preset)).into_result();
                let elapsed = start.elapsed();
                S_LOGGER.write_line(&format!("{file}: {}", format_duration(elapsed)));
                let has_diagnostics = result.is_err();
                // The C# condition is inverted (logs an error when there are
                // NO diagnostics) — ported verbatim (Program.cs:280).
                if !has_diagnostics {
                    S_LOGGER.log_error("Diagnostics were emitted.");
                }
            }
            Err(e) => S_LOGGER.log_error(&format!("Error reading {file}: {e}")),
        }
    }
}

/// Renders a duration like the dropped Tsu.Timing Duration.Format (the
/// hh:mm:ss.ffffff shape used by the CLI's prefix template).
fn format_duration(elapsed: std::time::Duration) -> String {
    let total = elapsed.as_micros();
    let us = total % 1_000_000;
    let total_s = total / 1_000_000;
    let s = total_s % 60;
    let total_m = total_s / 60;
    let m = total_m % 60;
    let h = total_m / 60;
    format!("{h:02}:{m:02}:{s:02}.{us:06}")
}

/// C# Program.MultiLua (Program.cs:364) — executes a file in every Lua
/// distribution via RunMultiLua.
fn multi_lua(script_path: &str) {
    run_multi_lua(&[script_path]);
}

/// C# Program.MultiLuaExpression (Program.cs:366-378) — writes the expression
/// to a temporary file and executes it via RunMultiLua.
fn multi_lua_expression(expression: &str) {
    let path = std::env::temp_dir().join(format!("loretta-cli-{}.lua", std::process::id()));
    std::fs::write(&path, expression).expect("write temp file");
    let path_str = path.to_string_lossy().into_owned();
    run_multi_lua(&[&path_str]);
    let _ = std::fs::remove_file(&path);
}

/// C# Program.RunMultiLua (Program.cs:380-433): executes the file in every
/// Lua distribution under binaries/. The output/error streaming order is
/// event-driven in C# (timing-dependent); the port drains stdout then stderr.
fn run_multi_lua(args: &[&str]) {
    const PREFIX_TEMPLATE: &str = "[00:00:00.000000]";
    let mut versions: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir("binaries") {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                versions.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
    }
    versions.sort();
    for version in versions {
        let name = version.replace('_', " ");
        let executable = std::path::Path::new("binaries")
            .join(&version)
            .join("lua.exe");
        // C# pads to Console.WindowWidth - prefix length; the console width
        // is runtime data, so the port uses the common 80-column default.
        let width = 80usize.saturating_sub(PREFIX_TEMPLATE.len());
        let mut title = format!("===== {name} ");
        while title.len() < width {
            title.push('=');
        }
        S_LOGGER.write_line(&title);
        let mut child = match std::process::Command::new(&executable)
            .args(args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .stdin(std::process::Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                S_LOGGER.log_error(&format!("Failed to start {executable:?}: {e}"));
                continue;
            }
        };
        // C# waits 2000ms then kills.
        let mut exited = false;
        for _ in 0..40 {
            match child.try_wait() {
                Ok(Some(_)) => {
                    exited = true;
                    break;
                }
                Ok(None) => std::thread::sleep(std::time::Duration::from_millis(50)),
                Err(e) => {
                    S_LOGGER.log_error(&format!("Error waiting for process: {e}"));
                    exited = true;
                    break;
                }
            }
        }
        if !exited {
            S_LOGGER.log_error("Process has timed out, killing...");
            let _ = child.kill();
            S_LOGGER.log_error("Killed.");
        }
        if let Some(stdout) = child.stdout.take() {
            let reader = std::io::BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                S_LOGGER.write_line(&line);
            }
        }
        if let Some(stderr) = child.stderr.take() {
            let reader = std::io::BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                S_LOGGER.log_error(&line);
            }
        }
        let _ = child.wait();
    }
}

/// C# Program.InvokeGc (Program.cs:497-504). The .NET GC.Collect /
/// WaitForPendingFinalizers calls are runtime infra with no Rust equivalent
/// (dropped); the loop structure is preserved.
fn invoke_gc(amount: i32) {
    for _ in 0..amount {
        // GC.Collect(GC.MaxGeneration, Forced, blocking, compacting);
        // GC.WaitForPendingFinalizers();
    }
}

/// C# Program.PopMemoryUsage (Program.cs:483-493): compares and pops the
/// most recent memory usage.
fn pop_memory_usage() {
    if S_MEMORY_STACK.lock().expect("memory stack lock").is_empty() {
        S_LOGGER.log_error("Nothing on memory stack to pop.");
        return;
    }
    compare_memory_usage();
    S_MEMORY_STACK.lock().expect("memory stack lock").pop();
}

/// C# Program.CompareMemoryUsage (Program.cs:462-481).
fn compare_memory_usage() {
    let curr_gc_mem = gc_memory();
    let curr_proc_mem = process_memory();
    S_LOGGER.write_line(&format!(
        "Memory usage according to GC:       {}",
        file_size_format(curr_gc_mem)
    ));
    S_LOGGER.write_line(&format!(
        "Memory usage according to Process:  {}",
        file_size_format(curr_proc_mem)
    ));
    let stack = S_MEMORY_STACK.lock().expect("memory stack lock");
    let Some((old_gc_mem, old_proc_mem)) = stack.last() else {
        drop(stack);
        S_LOGGER.log_error("Nothing on memory stack to compare to.");
        return;
    };
    let delta_gc = curr_gc_mem as i64 - *old_gc_mem as i64;
    let delta_proc = curr_proc_mem as i64 - *old_proc_mem as i64;
    S_LOGGER.write_line(&format!(
        "ΔMemory usage according to GC:      {}",
        if delta_gc < 0 {
            format!("-{}", file_size_format((-delta_gc) as u64))
        } else {
            file_size_format(delta_gc as u64)
        }
    ));
    S_LOGGER.write_line(&format!(
        "ΔMemory usage according to Process: {}",
        if delta_proc < 0 {
            format!("-{}", file_size_format((-delta_proc) as u64))
        } else {
            file_size_format(delta_proc as u64)
        }
    ));
}

/// C# Program.PushMemoryUsage (Program.cs:452-460): prints and pushes the
/// current memory usage.
fn push_memory_usage() {
    let gc_mem = gc_memory();
    let proc_mem = process_memory();
    S_LOGGER.write_line(&format!(
        "Memory usage according to GC:       {}",
        file_size_format(gc_mem)
    ));
    S_LOGGER.write_line(&format!(
        "Memory usage according to Process:  {}",
        file_size_format(proc_mem)
    ));
    S_MEMORY_STACK
        .lock()
        .expect("memory stack lock")
        .push((gc_mem, proc_mem));
    S_LOGGER.write_line("Memory usage pushed to stack.");
}

/// C# Program.PrintMemoryUsage (Program.cs:444-450).
fn print_memory_usage() {
    let gc_mem = gc_memory();
    let proc_mem = process_memory();
    S_LOGGER.write_line(&format!(
        "Memory usage according to GC:       {}",
        file_size_format(gc_mem)
    ));
    S_LOGGER.write_line(&format!(
        "Memory usage according to Process:  {}",
        file_size_format(proc_mem)
    ));
}

/// C# Program.Clear (Program.cs:437) — clears the console (ANSI escape
/// sequence; Console.Clear is platform-specific).
fn clear() {
    print!("\x1b[2J\x1b[H");
    io::stdout().flush().expect("flush stdout");
}

/// The Tsu.Timing LogLevel variants used by the CLI (dropped external enum).
#[derive(Copy, Clone)]
pub enum LogLevel {
    None,
    Error,
}

/// C# TimingLoggerConsole.Writer (TimingLoggerConsole.cs:24-40): routes
/// writes to the logger at a log level. The C# reflectively binds the
/// logger's private ProcessWrite; the port routes directly (dropped
/// reflection infra).
pub struct Writer<'a> {
    log_level: LogLevel,
    logger: &'a ConsoleTimingLogger,
}

impl Writer<'_> {
    /// C# Writer(TimingLogger, LogLevel).
    pub fn new(logger: &ConsoleTimingLogger, log_level: LogLevel) -> Writer<'_> {
        Writer { log_level, logger }
    }

    /// C# Writer.Write(string) — routes by level.
    pub fn write(&self, value: &str) {
        match self.log_level {
            LogLevel::None => self.logger.write_line(value),
            LogLevel::Error => self.logger.log_error(value),
        }
    }
}

/// C# TimingLoggerConsole (TimingLoggerConsole.cs:7-42): the System.CommandLine
/// IConsole adapter — the interface is dropped; the port keeps the
/// Out/Error writers and the redirection flags.
pub struct TimingLoggerConsole<'a> {
    out_writer: Writer<'a>,
    error_writer: Writer<'a>,
}

impl TimingLoggerConsole<'_> {
    /// C# TimingLoggerConsole(TimingLogger).
    pub fn new(logger: &ConsoleTimingLogger) -> TimingLoggerConsole<'_> {
        TimingLoggerConsole {
            out_writer: Writer::new(logger, LogLevel::None),
            error_writer: Writer::new(logger, LogLevel::Error),
        }
    }

    /// C# Out.
    pub fn out(&self) -> &Writer<'_> {
        &self.out_writer
    }

    /// C# IsOutputRedirected — always false.
    pub fn is_output_redirected(&self) -> bool {
        false
    }

    /// C# Error.
    pub fn error(&self) -> &Writer<'_> {
        &self.error_writer
    }

    /// C# IsErrorRedirected — always false.
    pub fn is_error_redirected(&self) -> bool {
        false
    }

    /// C# IsInputRedirected — always false.
    pub fn is_input_redirected(&self) -> bool {
        false
    }
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

/// C# Program.GetNamingStrategy (Program.cs:291-299): the enum to the
/// minifying naming strategy.
fn get_naming_strategy(
    naming_strategy: NamingStrategy,
) -> loretta::experimental::minifying::namingstrategy::NamingStrategy {
    use loretta::experimental::minifying::namingstrategies::NamingStrategies;
    match naming_strategy {
        NamingStrategy::Alphabetical => Box::new(NamingStrategies::alphabetical),
        NamingStrategy::Numerical => Box::new(NamingStrategies::numerical),
        NamingStrategy::ZeroWidth => Box::new(NamingStrategies::zero_width),
    }
}

/// C# Program.GetSlotAllocator (Program.cs:308-316).
fn get_slot_allocator(
    slot_allocator: SlotAllocator,
) -> Box<dyn loretta::experimental::minifying::islotallocator::ISlotAllocator> {
    use loretta::experimental::minifying::sequentialslotallocator::SequentialSlotAllocator;
    use loretta::experimental::minifying::sortedslotallocator::SortedSlotAllocator;
    match slot_allocator {
        SlotAllocator::Sequential => Box::new(SequentialSlotAllocator::new()),
        SlotAllocator::Sorted => Box::new(SortedSlotAllocator::new()),
    }
}

/// C# Program.Minify (Program.cs:318-360): minifies the provided file with
/// the naming strategy + slot allocator and writes the result.
fn minify(
    path: &str,
    preset: LuaSyntaxOptionsPreset,
    naming_strategy: NamingStrategy,
    slot_allocator: SlotAllocator,
    format: bool,
) {
    if !std::path::Path::new(path).is_file() {
        S_LOGGER.log_error("Provided path does not exist.");
        return;
    }
    let _ = (preset, format); // the preset maps to the parse version; the C# NormalizeWhitespace maps to the dropped formatter.
    let code = std::fs::read_to_string(path).expect("read file");
    let minified = loretta::experimental::luaextensions::minify_with(
        &code,
        get_naming_strategy(naming_strategy),
        get_slot_allocator(slot_allocator),
    );
    writeln!(output_writer(), "{minified}").expect("write output");
    writeln!(output_writer()).expect("write output");
}

/// C# Program.Parse (Program.cs:182-233): parses the provided file,
/// optionally constant-folds, and writes the code (or the tree dump).
fn parse(
    preset: LuaSyntaxOptionsPreset,
    path: &str,
    constant_fold: bool,
    print_tree: bool,
    assume_no_overrides: bool,
) {
    if !std::path::Path::new(path).is_file() {
        S_LOGGER.log_error("Provided path does not exist.");
        return;
    }
    let code = std::fs::read_to_string(path).expect("read file");
    let ast = full_moon::parse_fallible(&code, preset_to_lua_version(preset))
        .into_result()
        .expect("parse");
    let result = if constant_fold {
        loretta::experimental::luaextensions::constant_fold(
            ast,
            loretta::experimental::constantfoldingoptions::ConstantFoldingOptions {
                extract_numbers_from_strings: assume_no_overrides,
            },
        )
    } else {
        ast
    };
    if print_tree {
        S_LOGGER.write_line(&result.to_string());
    } else {
        writeln!(output_writer(), "{result}").expect("write output");
    }
}

/// C# Program.ParseExpression (Program.cs:235-...): parses the provided
/// expression (optionally preset-prefixed) and writes the result.
fn parse_expression(input: &str) {
    let (preset, code) = if let Some((head, rest)) = input.split_once(' ') {
        if let Some(p) = preset_from_name(head) {
            (p, rest.to_string())
        } else {
            (LuaSyntaxOptionsPreset::All, input.to_string())
        }
    } else {
        (LuaSyntaxOptionsPreset::All, input.to_string())
    };
    let ast = full_moon::parse_fallible(&code, preset_to_lua_version(preset))
        .into_result()
        .expect("parse");
    writeln!(output_writer(), "{ast}").expect("write output");
}

/// C# Enum.TryParse<LuaSyntaxOptionsPreset> — the preset name lookup.
fn preset_from_name(name: &str) -> Option<LuaSyntaxOptionsPreset> {
    match name.to_lowercase().as_str() {
        "lua51" => Some(LuaSyntaxOptionsPreset::Lua51),
        "lua52" => Some(LuaSyntaxOptionsPreset::Lua52),
        "lua53" => Some(LuaSyntaxOptionsPreset::Lua53),
        "lua54" => Some(LuaSyntaxOptionsPreset::Lua54),
        "luajit20" | "luajit2.0" => Some(LuaSyntaxOptionsPreset::LuaJit20),
        "luajit21" | "luajit2.1" => Some(LuaSyntaxOptionsPreset::LuaJit21),
        "gmod" => Some(LuaSyntaxOptionsPreset::GMod),
        "luau" => Some(LuaSyntaxOptionsPreset::Luau),
        "fivem" => Some(LuaSyntaxOptionsPreset::FiveM),
        "all" => Some(LuaSyntaxOptionsPreset::All),
        "allwithintegers" | "alli" => Some(LuaSyntaxOptionsPreset::Alli),
        _ => None,
    }
}

/// Splits the command-line args (System.CommandLine's whitespace split).
fn parse_args(args: &str) -> Vec<String> {
    args.split_whitespace().map(String::from).collect()
}

fn has_flag(args: &[String], flags: &[&str]) -> bool {
    args.iter().any(|a| flags.contains(&a.as_str()))
}

fn option_value(args: &[String], flags: &[&str]) -> Option<String> {
    for (i, a) in args.iter().enumerate() {
        if flags.contains(&a.as_str()) {
            return args.get(i + 1).cloned();
        }
    }
    None
}

fn first_positional(args: &[String]) -> String {
    args.iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_default()
}

/// The parse command handler (C# parseCommand with the -p/-c/-t/-a options).
fn parse_command_handler(args: &str) {
    let args = parse_args(args);
    let path = first_positional(&args);
    let preset = option_value(&args, &["-p", "--preset"])
        .and_then(|n| preset_from_name(&n))
        .unwrap_or(LuaSyntaxOptionsPreset::All);
    let constant_fold = has_flag(&args, &["-c", "--constant-fold"]);
    let print_tree = has_flag(&args, &["-t", "--print-tree"]);
    let assume_no_overrides = has_flag(&args, &["-a", "--assume-no-overrides"]);
    parse(
        preset,
        &path,
        constant_fold,
        print_tree,
        assume_no_overrides,
    );
}

/// The parse-expression command handler (C# parseExpressionCommand).
fn parse_expression_command_handler(args: &str) {
    let args = parse_args(args);
    let input = args.join(" ");
    parse_expression(&input);
}

/// The lex command handler (C# lexCommand with the -p/-t options).
fn lex_command_handler(args: &str) {
    let args = parse_args(args);
    let path = first_positional(&args);
    let preset = option_value(&args, &["-p", "--preset"])
        .and_then(|n| preset_from_name(&n))
        .unwrap_or(LuaSyntaxOptionsPreset::All);
    let print_tokens = has_flag(&args, &["-t", "--print-tokens"]);
    lex_command(preset, &path, print_tokens);
}

/// The minify command handler (C# minifyCommand with the -p/-n/-a/-f options).
fn minify_command_handler(args: &str) {
    let args = parse_args(args);
    let path = first_positional(&args);
    let preset = option_value(&args, &["-p", "--preset"])
        .and_then(|n| preset_from_name(&n))
        .unwrap_or(LuaSyntaxOptionsPreset::All);
    let naming = option_value(&args, &["-n", "--naming", "--naming-strategy"])
        .and_then(|n| match n.to_lowercase().as_str() {
            "alphabetical" => Some(NamingStrategy::Alphabetical),
            "numerical" => Some(NamingStrategy::Numerical),
            "zerowidth" => Some(NamingStrategy::ZeroWidth),
            _ => None,
        })
        .unwrap_or(NamingStrategy::Numerical);
    let allocator = option_value(&args, &["-a", "--allocator", "--slot-allocator"])
        .and_then(|n| match n.to_lowercase().as_str() {
            "sequential" => Some(SlotAllocator::Sequential),
            "sorted" => Some(SlotAllocator::Sorted),
            _ => None,
        })
        .unwrap_or(SlotAllocator::Sorted);
    let format = has_flag(&args, &["-f", "--format"]);
    minify(&path, preset, naming, allocator, format);
}

/// C# Program's static ctor (Program.cs:508-...): the root command table.
fn build_root_command() -> Vec<Command> {
    vec![
        Command {
            name: "@cd",
            aliases: &[],
            handler: |args| change_directory(args.trim()),
        },
        Command {
            name: "s",
            aliases: &["set"],
            handler: |args| {
                let parts: Vec<&str> = args.split_whitespace().collect();
                if parts.len() < 2 {
                    S_LOGGER.log_error("s: expected <setting> <value>");
                    return;
                }
                let setting = match parts[0] {
                    "@cd" | "printcurrentdir" | "printcurrentdirectory" => Setting::PrintCurrentDir,
                    "p" | "printoutputprefixed" => Setting::PrintOutputPrefixed,
                    _ => {
                        S_LOGGER.log_error(&format!("Invalid setting '{}'.", parts[0]));
                        return;
                    }
                };
                if let Err(e) = set_setting(setting, parts[1]) {
                    S_LOGGER.log_error(&e);
                }
            },
        },
        Command {
            name: "q",
            aliases: &["quit", "exit"],
            handler: |_| quit(),
        },
        Command {
            name: "cd",
            aliases: &[],
            handler: |args| change_directory(args.trim()),
        },
        Command {
            name: "ls",
            aliases: &["list"],
            handler: |_| list_symbols(),
        },
        Command {
            name: "l",
            aliases: &["lex"],
            handler: lex_command_handler,
        },
        Command {
            name: "p",
            aliases: &["parse"],
            handler: parse_command_handler,
        },
        Command {
            name: "e",
            aliases: &["expr", "expression"],
            handler: parse_expression_command_handler,
        },
        Command {
            name: "min",
            aliases: &["minify"],
            handler: minify_command_handler,
        },
        Command {
            name: "mp",
            aliases: &["mass-parse"],
            handler: |args| {
                let patterns: Vec<&str> = args.split_whitespace().collect();
                if patterns.is_empty() {
                    S_LOGGER.log_error("mp: expected at least one pattern");
                    return;
                }
                mass_parse_command(LuaSyntaxOptionsPreset::All, &patterns);
            },
        },
        Command {
            name: "mlua",
            aliases: &["multi-lua", "multilua"],
            handler: |args| multi_lua(args.trim()),
        },
        Command {
            name: "emlua",
            aliases: &["execute-multi-lua"],
            handler: |args| multi_lua_expression(args.trim()),
        },
        Command {
            name: "c",
            aliases: &["clear"],
            handler: |_| clear(),
        },
        Command {
            name: "mem",
            aliases: &["memory"],
            handler: |_| print_memory_usage(),
        },
        Command {
            name: "mem+",
            aliases: &["push-memory"],
            handler: |_| push_memory_usage(),
        },
        Command {
            name: "mem-",
            aliases: &["pop-memory"],
            handler: |_| pop_memory_usage(),
        },
        Command {
            name: "memcmp",
            aliases: &["compare-memory"],
            handler: |_| compare_memory_usage(),
        },
        Command {
            name: "gc",
            aliases: &[],
            handler: |args| {
                let n = args.trim().parse::<i32>().unwrap_or(0);
                invoke_gc(n);
            },
        },
        Command {
            name: "help",
            aliases: &["h", "?"],
            handler: |_| {
                S_LOGGER.write_line("Commands:");
                for command in S_ROOT_COMMAND.get().expect("root command built") {
                    S_LOGGER.write_line(&format!(
                        "  {}{}",
                        command.name,
                        if command.aliases.is_empty() {
                            String::new()
                        } else {
                            format!(" ({})", command.aliases.join(", "))
                        }
                    ));
                }
            },
        },
    ]
}

fn main() {
    // C# Main (Program.cs:25-...): the REPL loop. The static ctor (row 456)
    // maps to the OnceLock initialization.
    let root = S_ROOT_COMMAND.get_or_init(build_root_command);
    S_SHOULD_RUN.store(true, Ordering::Relaxed);
    // C# Main: s_currentProc (the process for the memory command).
    let _ = current_proc;
    while S_SHOULD_RUN.load(Ordering::Relaxed) {
        if S_PRINT_CURRENT_DIR.load(Ordering::Relaxed) {
            if let Ok(dir) = std::env::current_dir() {
                S_LOGGER.write(&dir.display().to_string());
            }
        }
        S_LOGGER.write("> ");
        let mut line = String::new();
        if std::io::stdin()
            .read_line(&mut line)
            .map(|n| n == 0)
            .unwrap_or(true)
        {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (name, args) = match line.find(char::is_whitespace) {
            Some(i) => (&line[..i], &line[i + 1..]),
            None => (line, ""),
        };
        match root
            .iter()
            .find(|c| c.name == name || c.aliases.contains(&name))
        {
            Some(command) => (command.handler)(args),
            None => S_LOGGER.log_error(&format!("Invalid command '{name}'.")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Oracle: the reference CLI's `l --print-tokens` output for
    /// corpus/features/charutils.lua (captured from the reference package's
    /// ParseTokens + LuaTreeDumperConverter + TreeDumper.DumpCompact).
    #[test]
    fn lex_dump_matches_reference() {
        let code = include_str!("../../corpus/features/charutils.lua");
        let ast = full_moon::parse_fallible(code, full_moon::LuaVersion::new())
            .into_result()
            .expect("parse");
        let tokens = collect_tokens(&ast);
        assert_eq!(tokens.len(), 25);
        assert_eq!(
            build_token_dump(&tokens),
            r#"Root
├─LocalKeyword: local
│ ├─Leading Trivia
│ │ ├─SingleLineCommentTrivia: -- CharUtils: IsAlpha, IsBinary, IsDecimal, IsHexadecimal, IsWhitespace, IsValidFirst/TrailingIdentifierChar
│ │ │ └─Diagnostics
│ │ └─EndOfLineTrivia: \n
│ │   └─Diagnostics
│ ├─Trailing Trivia
│ │ └─WhitespaceTrivia:  
│ │   └─Diagnostics
│ └─Diagnostics
├─IdentifierToken: _a
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─WhitespaceTrivia:  
│ │   └─Diagnostics
│ └─Diagnostics
├─EqualsToken: =
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─WhitespaceTrivia:  
│ │   └─Diagnostics
│ └─Diagnostics
├─NumericLiteralToken: 1
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─EndOfLineTrivia: \n
│ │   └─Diagnostics
│ └─Diagnostics
├─LocalKeyword: local
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─WhitespaceTrivia:  
│ │   └─Diagnostics
│ └─Diagnostics
├─IdentifierToken: a1
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─WhitespaceTrivia:  
│ │   └─Diagnostics
│ └─Diagnostics
├─EqualsToken: =
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─WhitespaceTrivia:  
│ │   └─Diagnostics
│ └─Diagnostics
├─NumericLiteralToken: 2
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─EndOfLineTrivia: \n
│ │   └─Diagnostics
│ └─Diagnostics
├─LocalKeyword: local
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─WhitespaceTrivia:  
│ │   └─Diagnostics
│ └─Diagnostics
├─IdentifierToken: __test
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─WhitespaceTrivia:  
│ │   └─Diagnostics
│ └─Diagnostics
├─EqualsToken: =
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─WhitespaceTrivia:  
│ │   └─Diagnostics
│ └─Diagnostics
├─NumericLiteralToken: 3
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─EndOfLineTrivia: \n
│ │   └─Diagnostics
│ └─Diagnostics
├─LocalKeyword: local
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─WhitespaceTrivia:  
│ │   └─Diagnostics
│ └─Diagnostics
├─IdentifierToken: a
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─WhitespaceTrivia:  
│ │   └─Diagnostics
│ └─Diagnostics
├─EqualsToken: =
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─WhitespaceTrivia:  
│ │   └─Diagnostics
│ └─Diagnostics
├─NumericLiteralToken: 10
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─EndOfLineTrivia: \n
│ │   └─Diagnostics
│ └─Diagnostics
├─LocalKeyword: local
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─WhitespaceTrivia:  
│ │   └─Diagnostics
│ └─Diagnostics
├─IdentifierToken: b
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─WhitespaceTrivia:  
│ │   └─Diagnostics
│ └─Diagnostics
├─EqualsToken: =
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─WhitespaceTrivia:  
│ │   └─Diagnostics
│ └─Diagnostics
├─NumericLiteralToken: 26
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─EndOfLineTrivia: \n
│ │   └─Diagnostics
│ └─Diagnostics
├─LocalKeyword: local
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─WhitespaceTrivia:  
│ │   └─Diagnostics
│ └─Diagnostics
├─IdentifierToken: c
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─WhitespaceTrivia:  
│ │   └─Diagnostics
│ └─Diagnostics
├─EqualsToken: =
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─WhitespaceTrivia:  
│ │   └─Diagnostics
│ └─Diagnostics
├─NumericLiteralToken: 123
│ ├─Leading Trivia
│ ├─Trailing Trivia
│ │ └─EndOfLineTrivia: \n
│ │   └─Diagnostics
│ └─Diagnostics
└─EndOfFileToken: 
  ├─Leading Trivia
  ├─Trailing Trivia
  └─Diagnostics"#
        );
    }
}
