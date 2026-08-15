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

/// C# Program.Setting (private enum).
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
/// patterns. The enumeration order is unspecified in C#, so the port sorts
/// the file list for a deterministic oracle; the duration output is runtime
/// data (not byte-comparable).
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
    files.sort();
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
        LuaSyntaxOptionsPreset::LuaJit20,
        LuaSyntaxOptionsPreset::LuaJit21,
        LuaSyntaxOptionsPreset::GMod,
        LuaSyntaxOptionsPreset::Luau,
        LuaSyntaxOptionsPreset::FiveM,
        LuaSyntaxOptionsPreset::All,
        LuaSyntaxOptionsPreset::Alli,
    );
    // Referenced until Lex (row 431) uses it.
    let _ = preset_enum_to_preset_options
        as fn(LuaSyntaxOptionsPreset) -> loretta::luaparseoptions::LuaParseOptions;
    // Constructed until GetSlotAllocator (row 443) uses it.
    let _ = (SlotAllocator::Sequential, SlotAllocator::Sorted);
    // Constructed until GetNamingStrategy (row 439) uses it.
    let _ = (
        NamingStrategy::Alphabetical,
        NamingStrategy::Numerical,
        NamingStrategy::ZeroWidth,
    );
    // Referenced until the static ctor (row 456) wires the lex command.
    let _ = lex_command as fn(LuaSyntaxOptionsPreset, &str, bool);
    // Referenced until the static ctor (row 456) wires the mass-parse command.
    let _ = mass_parse_command as fn(LuaSyntaxOptionsPreset, &[&str]);
    // Referenced until the static ctor (row 456) wires the multi-lua command.
    let _ = run_multi_lua as fn(&[&str]);
    let _ = multi_lua as fn(&str);
    let _ = multi_lua_expression as fn(&str);
    let _ = clear as fn();
    // Referenced until the static ctor (row 456) wires the memory command.
    let _ = print_memory_usage as fn();
    let _ = push_memory_usage as fn();
    // Referenced until the memory rows (451-455) land.
    let _ = current_proc as fn() -> u32;
    let _ = (gc_memory as fn() -> u64, process_memory as fn() -> u64);
    let _ = file_size_format as fn(u64) -> String;
    let _ = &S_MEMORY_STACK;
    writeln!(
        output_writer(),
        "loretta-cli: pending port — see loretta-rs/PROGRESS.md"
    )
    .expect("write output");
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
