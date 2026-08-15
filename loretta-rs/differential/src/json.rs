// Byte-exact replication of System.Text.Json `WriteIndented` output with the
// default JavaScriptEncoder, as produced by tools/differential/Program.cs:
//   - 2-space indent, `"key": value` on its own line
//   - strings escape `"` `'` `&` `<` `>` `+` and all non-ASCII / control chars as \uXXXX
//     (only \n \r \t \b \f \\ use their short JSON forms)
//   - no trailing newline (File.WriteAllTextAsync semantics)

pub enum Json {
    Object(Vec<(String, Json)>),
    Array(Vec<Json>),
    String(String),
    Number(i64),
    Bool(bool),
}

impl From<i32> for Json {
    fn from(v: i32) -> Self {
        Json::Number(v as i64)
    }
}

impl From<usize> for Json {
    fn from(v: usize) -> Self {
        Json::Number(v as i64)
    }
}

pub fn render(root: &Json) -> Vec<u8> {
    let mut out = Vec::new();
    write_value(root, 0, &mut out);
    out
}

fn write_value(value: &Json, depth: usize, out: &mut Vec<u8>) {
    match value {
        Json::Object(entries) => {
            if entries.is_empty() {
                out.extend_from_slice(b"{}");
                return;
            }
            out.push(b'{');
            for (i, (key, val)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                out.push(b'\n');
                indent(depth + 1, out);
                write_string(key, out);
                out.extend_from_slice(b": ");
                write_value(val, depth + 1, out);
            }
            out.push(b'\n');
            indent(depth, out);
            out.push(b'}');
        }
        Json::Array(items) => {
            if items.is_empty() {
                out.extend_from_slice(b"[]");
                return;
            }
            out.push(b'[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                out.push(b'\n');
                indent(depth + 1, out);
                write_value(item, depth + 1, out);
            }
            out.push(b'\n');
            indent(depth, out);
            out.push(b']');
        }
        Json::String(s) => write_string(s, out),
        Json::Number(n) => out.extend_from_slice(n.to_string().as_bytes()),
        Json::Bool(b) => out.extend_from_slice(if *b { b"true" } else { b"false" }),
    }
}

fn indent(depth: usize, out: &mut Vec<u8>) {
    out.resize(out.len() + depth * 2, b' ');
}

fn write_string(s: &str, out: &mut Vec<u8>) {
    out.push(b'"');
    for c in s.chars() {
        let code = c as u32;
        match c {
            '"' | '\'' | '&' | '<' | '>' | '+' => {
                out.extend_from_slice(format!("\\u{code:04X}").as_bytes());
            }
            '\\' => out.extend_from_slice(b"\\\\"),
            '\n' => out.extend_from_slice(b"\\n"),
            '\r' => out.extend_from_slice(b"\\r"),
            '\t' => out.extend_from_slice(b"\\t"),
            '\u{8}' => out.extend_from_slice(b"\\b"),
            '\u{c}' => out.extend_from_slice(b"\\f"),
            _ if !(0x20..=0x7E).contains(&code) => {
                out.extend_from_slice(format!("\\u{code:04X}").as_bytes());
            }
            _ => out.extend_from_slice(c.encode_utf8(&mut [0u8; 4]).as_bytes()),
        }
    }
    out.push(b'"');
}
