use std::convert::TryFrom;
use std::str::FromStr;

use base64::{Engine, engine::general_purpose::STANDARD as BASE64_STANDARD};
use chrono::{Duration as ChronoDuration, NaiveDate, NaiveDateTime, TimeZone, Utc};
use mongodb::bson::spec::BinarySubtype;
use mongodb::bson::{
    Binary, Bson, DateTime, Decimal128, Document, JavaScriptCodeWithScope, Regex,
    Timestamp as BsonTimestamp, oid::ObjectId,
};
use serde_json::Value;
use uuid::Uuid;

use crate::i18n::{tr, tr_format};

pub fn format_bson_scalar(value: &Bson) -> (String, String) {
    match value {
        Bson::String(s) => (s.clone(), String::from(tr("String"))),
        Bson::Boolean(b) => (b.to_string(), String::from(tr("Boolean"))),
        Bson::Int32(i) => (i.to_string(), String::from(tr("Int32"))),
        Bson::Int64(i) => (i.to_string(), String::from(tr("Int64"))),
        Bson::Double(f) => {
            if f.is_finite() {
                (format!("{f}"), String::from(tr("Double")))
            } else {
                (format!("Double({f})"), String::from(tr("Double")))
            }
        }
        Bson::Decimal128(d) => {
            (format!("numberDecimal(\"{}\")", d), String::from(tr("Decimal128")))
        }
        Bson::DateTime(dt) => match dt.try_to_rfc3339_string() {
            Ok(iso) => (iso, String::from(tr("DateTime"))),
            Err(_) => {
                (format!("DateTime({})", dt.timestamp_millis()), String::from(tr("DateTime")))
            }
        },
        Bson::ObjectId(oid) => (format!("ObjectId(\"{}\")", oid), String::from(tr("ObjectId"))),
        Bson::Binary(bin) => (
            format!("Binary(len={}, subtype={:?})", bin.bytes.len(), bin.subtype),
            String::from(tr("Binary")),
        ),
        Bson::Symbol(sym) => (format!("Symbol({sym:?})"), String::from(tr("Symbol"))),
        Bson::RegularExpression(regex) => {
            if regex.options.is_empty() {
                (format!("Regex({:?})", regex.pattern), String::from(tr("Regex")))
            } else {
                (
                    format!("Regex({:?}, {:?})", regex.pattern, regex.options),
                    String::from(tr("Regex")),
                )
            }
        }
        Bson::Timestamp(ts) => (
            format!("Timestamp(time={}, increment={})", ts.time, ts.increment),
            String::from(tr("Timestamp")),
        ),
        Bson::JavaScriptCode(code) => {
            (format!("Code({code:?})"), String::from(tr("JavaScriptCode")))
        }
        Bson::JavaScriptCodeWithScope(code_with_scope) => {
            let scope_len = code_with_scope.scope.len();
            (
                format!("CodeWithScope({:?}, scope_fields={})", code_with_scope.code, scope_len),
                String::from(tr("JavaScriptCodeWithScope")),
            )
        }
        Bson::DbPointer(ptr) => (format!("DbPointer({ptr:?})"), String::from(tr("DbPointer"))),
        Bson::Undefined => (String::from(tr("undefined")), String::from(tr("Undefined"))),
        Bson::Null => (String::from(tr("null")), String::from(tr("Null"))),
        Bson::MinKey => (String::from(tr("MinKey")), String::from(tr("MinKey"))),
        Bson::MaxKey => (String::from(tr("MaxKey")), String::from(tr("MaxKey"))),
        Bson::Document(_) | Bson::Array(_) => unreachable!("containers handled separately"),
    }
}

pub fn format_bson_shell(value: &Bson) -> String {
    format_bson_shell_internal(value, 0)
}

fn format_bson_shell_internal(value: &Bson, level: usize) -> String {
    match value {
        Bson::Document(doc) => format_document_shell(doc, level),
        Bson::Array(items) => format_array_shell(items, level),
        _ => format_bson_shell_scalar(value),
    }
}

fn format_document_shell(doc: &Document, level: usize) -> String {
    if doc.is_empty() {
        return String::from(tr("{}"));
    }

    let indent_current = shell_indent(level);
    let indent_child = shell_indent(level + 1);

    let mut entries: Vec<Vec<String>> = Vec::new();
    for (key, value) in doc.iter() {
        let value_repr = format_bson_shell_internal(value, level + 1);
        let value_lines: Vec<&str> = value_repr.lines().collect();
        let mut lines = Vec::new();
        if let Some((first, rest)) = value_lines.split_first() {
            lines.push(format!("{indent_child}\"{key}\": {first}"));
            for line in rest {
                lines.push(line.to_string());
            }
        } else {
            lines.push(format!("{indent_child}\"{key}\": null"));
        }
        entries.push(lines);
    }

    let mut result = String::from(tr("{\n"));
    let entry_count = entries.len();
    for (index, mut entry) in entries.into_iter().enumerate() {
        if let Some(last) = entry.last_mut() {
            if index + 1 != entry_count {
                last.push(',');
            }
        }
        for line in entry {
            result.push_str(&line);
            result.push('\n');
        }
    }
    result.push_str(&indent_current);
    result.push('}');
    result
}

fn format_array_shell(items: &[Bson], level: usize) -> String {
    if items.is_empty() {
        return String::from(tr("[]"));
    }

    let indent_current = shell_indent(level);
    let indent_child = shell_indent(level + 1);

    let mut result = String::from(tr("[\n"));
    let len = items.len();
    for (index, item) in items.iter().enumerate() {
        let value_repr = format_bson_shell_internal(item, level + 1);
        let value_lines: Vec<&str> = value_repr.lines().collect();
        let last_line_index = value_lines.len().saturating_sub(1);
        for (line_index, line) in value_lines.into_iter().enumerate() {
            if line_index == 0 {
                result.push_str(&indent_child);
                result.push_str(line);
            } else {
                result.push_str(line);
            }
            if line_index == last_line_index && index + 1 != len {
                result.push(',');
            }
            result.push('\n');
        }
    }
    result.push_str(&indent_current);
    result.push(']');
    result
}

fn format_bson_shell_scalar(value: &Bson) -> String {
    match value {
        Bson::String(s) => serde_json::to_string(s).unwrap_or_else(|_| format!("\"{}\"", s)),
        Bson::Boolean(b) => b.to_string(),
        Bson::Int32(i) => i.to_string(),
        Bson::Int64(i) => i.to_string(),
        Bson::Double(f) => {
            if f.is_nan() {
                String::from(tr("NaN"))
            } else if f.is_infinite() {
                if f.is_sign_negative() {
                    String::from(tr("-Infinity"))
                } else {
                    String::from(tr("Infinity"))
                }
            } else {
                format!("{f}")
            }
        }
        Bson::Decimal128(d) => format!("NumberDecimal(\"{}\")", d),
        Bson::DateTime(dt) => match dt.try_to_rfc3339_string() {
            Ok(iso) => format!("ISODate(\"{}\")", iso),
            Err(_) => format!("DateTime({})", dt.timestamp_millis()),
        },
        Bson::ObjectId(oid) => format!("ObjectId(\"{}\")", oid),
        Bson::Binary(bin) => {
            if bin.subtype == BinarySubtype::Uuid && bin.bytes.len() == 16 {
                if let Ok(uuid) = Uuid::from_slice(&bin.bytes) {
                    format!("UUID(\"{}\")", uuid)
                } else {
                    let encoded = BASE64_STANDARD.encode(&bin.bytes);
                    let subtype: u8 = bin.subtype.into();
                    format!("BinData({}, \"{}\")", subtype, encoded)
                }
            } else {
                let encoded = BASE64_STANDARD.encode(&bin.bytes);
                let subtype: u8 = bin.subtype.into();
                format!("BinData({}, \"{}\")", subtype, encoded)
            }
        }
        Bson::Symbol(sym) => {
            let text = serde_json::to_string(sym).unwrap_or_else(|_| format!("\"{}\"", sym));
            format!("Symbol({text})")
        }
        Bson::RegularExpression(regex) => {
            let pattern = serde_json::to_string(&regex.pattern)
                .unwrap_or_else(|_| format!("\"{}\"", regex.pattern));
            let options = serde_json::to_string(&regex.options)
                .unwrap_or_else(|_| format!("\"{}\"", regex.options));
            format!("RegExp({pattern}, {options})")
        }
        Bson::Timestamp(ts) => format!("Timestamp({}, {})", ts.time, ts.increment),
        Bson::JavaScriptCode(code) => {
            let text = serde_json::to_string(code).unwrap_or_else(|_| format!("\"{}\"", code));
            format!("Code({text})")
        }
        Bson::JavaScriptCodeWithScope(code_with_scope) => {
            let code_text = serde_json::to_string(&code_with_scope.code)
                .unwrap_or_else(|_| format!("\"{}\"", code_with_scope.code));
            let scope = format_shell_value(&Bson::Document(code_with_scope.scope.clone()));
            format!("Code({code_text}, {scope})")
        }
        Bson::DbPointer(_) => serde_json::to_string(value)
            .unwrap_or_else(|_| String::from(tr("{\"$dbPointer\":{...}}"))),
        Bson::Undefined => String::from(tr("undefined")),
        Bson::Null => String::from(tr("null")),
        Bson::MinKey => String::from(tr("MinKey()")),
        Bson::MaxKey => String::from(tr("MaxKey()")),
        Bson::Document(_) | Bson::Array(_) => unreachable!("containers handled separately"),
    }
}

fn shell_indent(level: usize) -> String {
    const INDENT: usize = 4;
    " ".repeat(level * INDENT)
}

pub fn split_arguments(args: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut string_delim: Option<char> = None;
    let mut escape = false;
    let mut stack: Vec<char> = Vec::new();

    for ch in args.chars() {
        if let Some(delim) = string_delim {
            current.push(ch);
            if escape {
                escape = false;
                continue;
            }
            match ch {
                '\\' => escape = true,
                c if c == delim => string_delim = None,
                _ => {}
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                string_delim = Some(ch);
                current.push(ch);
            }
            '(' | '{' | '[' => {
                stack.push(ch);
                current.push(ch);
            }
            ')' | '}' | ']' => {
                if let Some(open) = stack.pop() {
                    if !matches!((open, ch), ('(', ')') | ('{', '}') | ('[', ']')) {
                        stack.clear();
                    }
                }
                current.push(ch);
            }
            ',' if stack.is_empty() => {
                result.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }

    result
}

pub fn parse_shell_document(source: &str) -> Result<Bson, String> {
    let value = parse_shell_bson_value(source)?;
    match value {
        Bson::Document(_) => Ok(value),
        other => Err(format!("{} {:?}", tr("Expected a document, got"), other)),
    }
}

pub fn parse_shell_array(source: &str) -> Result<Bson, String> {
    let value = parse_shell_bson_value(source)?;
    match value {
        Bson::Array(_) => Ok(value),
        other => Err(format!("{} {:?}", tr("Expected an array, got"), other)),
    }
}

pub fn parse_shell_json_value(source: &str) -> Result<Value, String> {
    let normalized = preprocess_shell_json(source)?;
    serde_json::from_str(&normalized).map_err(|error| format!("JSON parse error: {error}"))
}

fn preprocess_shell_json(source: &str) -> Result<String, String> {
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut result = String::with_capacity(source.len());
    let mut index = 0usize;

    while index < len {
        let ch = chars[index];

        if ch == '\"' {
            let end = skip_double_quoted(&chars, index)?;
            result.extend(&chars[index..end]);
            index = end;
            continue;
        }

        if ch == '\'' {
            let (json_literal, next_index) = collect_single_quoted_string(&chars, index)?;
            result.push_str(&json_literal);
            index = next_index;
            continue;
        }

        if ch == '-' {
            if let Some((replacement, consumed)) = try_parse_negative_constant(&chars, index)? {
                result.push_str(&replacement);
                index = consumed;
                continue;
            }
        }

        if ch == '/' {
            if let Some((replacement, consumed)) = try_parse_regex_literal(&chars, index)? {
                result.push_str(&replacement);
                index = consumed;
                continue;
            }
        }

        if is_identifier_start(ch) {
            let start_index = index;
            let (identifier, mut next_index) = read_identifier(&chars, index);
            index = next_index;

            if identifier == "new" {
                next_index = skip_whitespace(&chars, next_index);
                let (next_identifier, after_identifier) = read_identifier(&chars, next_index);
                if !next_identifier.is_empty() && is_special_construct(&next_identifier) {
                    if let Some((replacement, consumed)) =
                        convert_special_construct(&chars, after_identifier, &next_identifier)?
                    {
                        result.push_str(&replacement);
                        index = consumed;
                        continue;
                    }
                }

                result.push_str("new");
                if !next_identifier.is_empty() {
                    result.push(' ');
                    result.push_str(&next_identifier);
                    index = after_identifier;
                }
                continue;
            }

            if identifier == "function" {
                let (code, consumed) = extract_function_literal(&chars, start_index)?;
                let replacement = bson_to_extended_json(Bson::JavaScriptCode(code))?;
                result.push_str(&replacement);
                index = consumed;
                continue;
            }

            if let Some(replacement) = convert_constant(&identifier)? {
                result.push_str(&replacement);
                continue;
            }

            if is_special_construct(&identifier) {
                if let Some((replacement, consumed_until)) =
                    convert_special_construct(&chars, index, &identifier)?
                {
                    result.push_str(&replacement);
                    index = consumed_until;
                    continue;
                }
            }

            result.push_str(&identifier);
            continue;
        }

        result.push(ch);
        index += 1;
    }

    Ok(result)
}

fn skip_whitespace(chars: &[char], mut index: usize) -> usize {
    let len = chars.len();
    while index < len && chars[index].is_whitespace() {
        index += 1;
    }
    index
}

fn read_identifier(chars: &[char], start: usize) -> (String, usize) {
    let len = chars.len();
    if start >= len || !is_identifier_start(chars[start]) {
        return (String::new(), start);
    }
    let mut index = start + 1;
    while index < len && is_identifier_part(chars[index]) {
        index += 1;
    }
    (chars[start..index].iter().collect(), index)
}

fn convert_constant(identifier: &str) -> Result<Option<String>, String> {
    match identifier {
        "Infinity" => Ok(Some(bson_to_extended_json(Bson::Double(f64::INFINITY))?)),
        "NaN" => Ok(Some(bson_to_extended_json(Bson::Double(f64::NAN))?)),
        "undefined" => Ok(Some(bson_to_extended_json(Bson::Undefined)?)),
        _ => Ok(None),
    }
}

fn matches_keyword(chars: &[char], start: usize, keyword: &str) -> bool {
    let len = chars.len();
    let keyword_len = keyword.len();
    if start + keyword_len > len {
        return false;
    }

    chars[start..start + keyword_len].iter().zip(keyword.chars()).all(|(&ch, kw)| ch == kw)
}

fn prev_non_whitespace(chars: &[char], index: usize) -> Option<char> {
    let mut idx = index;
    while idx > 0 {
        idx -= 1;
        let ch = chars[idx];
        if !ch.is_whitespace() {
            return Some(ch);
        }
    }
    None
}

fn try_parse_negative_constant(
    chars: &[char],
    index: usize,
) -> Result<Option<(String, usize)>, String> {
    if matches_keyword(chars, index + 1, "Infinity") {
        let consumed = index + 1 + "Infinity".len();
        let replacement = bson_to_extended_json(Bson::Double(f64::NEG_INFINITY))?;
        return Ok(Some((replacement, consumed)));
    }

    Ok(None)
}

fn try_parse_regex_literal(
    chars: &[char],
    index: usize,
) -> Result<Option<(String, usize)>, String> {
    if chars[index] != '/' {
        return Ok(None);
    }

    if let Some(prev) = prev_non_whitespace(chars, index) {
        if !matches!(prev, ':' | ',' | '{' | '[' | '(') {
            return Ok(None);
        }
    }

    let len = chars.len();
    let mut pattern = String::new();
    let mut escape = false;
    let mut cursor = index + 1;

    while cursor < len {
        let ch = chars[cursor];
        if escape {
            pattern.push(ch);
            escape = false;
        } else if ch == '\\' {
            pattern.push(ch);
            escape = true;
        } else if ch == '/' {
            break;
        } else {
            pattern.push(ch);
        }
        cursor += 1;
    }

    if cursor >= len || chars[cursor] != '/' {
        return Err(String::from(tr("Regular expression is not terminated with '/'.")));
    }

    cursor += 1;
    let mut options = String::new();
    while cursor < len && chars[cursor].is_ascii_alphabetic() {
        options.push(chars[cursor]);
        cursor += 1;
    }

    let regex = Regex { pattern, options };
    let replacement = bson_to_extended_json(Bson::RegularExpression(regex))?;
    Ok(Some((replacement, cursor)))
}

fn extract_function_literal(chars: &[char], start: usize) -> Result<(String, usize), String> {
    let len = chars.len();
    let mut index = start;
    let mut buffer = String::new();
    let mut in_string = false;
    let mut string_delim = '\'';
    let mut escape = false;
    let mut paren_depth = 0i32;
    let mut brace_depth = 0i32;
    let mut encountered_brace = false;

    while index < len {
        let ch = chars[index];
        buffer.push(ch);
        index += 1;

        if in_string {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
            } else if ch == string_delim {
                in_string = false;
            }
            continue;
        }

        match ch {
            '\'' | '"' => {
                in_string = true;
                string_delim = ch;
            }
            '(' => paren_depth += 1,
            ')' => {
                if paren_depth > 0 {
                    paren_depth -= 1;
                }
            }
            '{' => {
                brace_depth += 1;
                encountered_brace = true;
            }
            '}' => {
                if brace_depth > 0 {
                    brace_depth -= 1;
                    if encountered_brace && brace_depth == 0 && paren_depth == 0 {
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    if brace_depth != 0 {
        return Err(String::from(tr("Function is missing a closing brace.")));
    }

    Ok((buffer.trim().to_string(), index))
}

fn collect_single_quoted_string(chars: &[char], start: usize) -> Result<(String, usize), String> {
    let (raw, next_index) = read_single_quoted(chars, start)?;
    Ok((Value::String(raw).to_string(), next_index))
}

fn read_single_quoted(chars: &[char], start: usize) -> Result<(String, usize), String> {
    let mut buffer = String::new();
    let mut index = start + 1;
    let len = chars.len();

    while index < len {
        match chars[index] {
            '\\' => {
                index += 1;
                if index >= len {
                    return Err(String::from(tr(
                        "Single-quoted string contains an unfinished escape sequence.",
                    )));
                }

                let (ch, consumed) = match chars[index] {
                    '\\' => ('\\', 1),
                    '\'' => ('\'', 1),
                    '"' => ('"', 1),
                    'n' => ('\n', 1),
                    'r' => ('\r', 1),
                    't' => ('\t', 1),
                    'b' => ('\u{0008}', 1),
                    'f' => ('\u{000C}', 1),
                    'v' => ('\u{000B}', 1),
                    '0' => ('\u{0000}', 1),
                    'x' => {
                        if index + 2 >= len {
                            return Err(String::from(tr(
                                "The \\x sequence must contain two hex digits.",
                            )));
                        }
                        let high = hex_value(chars[index + 1])?;
                        let low = hex_value(chars[index + 2])?;
                        let value = ((high << 4) | low) as u32;
                        (codepoint_to_char(value)?, 3)
                    }
                    'u' => {
                        if index + 4 >= len {
                            return Err(String::from(tr(
                                "The \\u sequence must contain four hex digits.",
                            )));
                        }
                        let mut value = 0u32;
                        for offset in 1..=4 {
                            value = (value << 4) | hex_value(chars[index + offset])?;
                        }
                        (codepoint_to_char(value)?, 5)
                    }
                    other => (other, 1),
                };

                buffer.push(ch);
                index += consumed;
            }
            '\'' => return Ok((buffer, index + 1)),
            other => {
                buffer.push(other);
                index += 1;
            }
        }
    }

    Err(String::from(tr("Single-quoted string is not closed.")))
}

fn skip_single_quoted(chars: &[char], start: usize) -> Result<usize, String> {
    let (_, next) = read_single_quoted(chars, start)?;
    Ok(next)
}

fn skip_double_quoted(chars: &[char], start: usize) -> Result<usize, String> {
    let mut index = start + 1;
    let len = chars.len();

    while index < len {
        match chars[index] {
            '\\' => {
                index += 2;
            }
            '\"' => return Ok(index + 1),
            _ => index += 1,
        }
    }

    Err(String::from(tr("Double-quoted string is not closed.")))
}

fn hex_value(ch: char) -> Result<u32, String> {
    ch.to_digit(16).ok_or_else(|| {
        tr_format("Invalid hex character '{}' in escape sequence.", &[&ch.to_string()])
    })
}

fn codepoint_to_char(value: u32) -> Result<char, String> {
    char::from_u32(value).ok_or_else(|| {
        tr_format("Code point 0x{} is not a valid character.", &[&format!("{value:04X}")])
    })
}

fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_identifier_part(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '.'
}

fn is_special_construct(identifier: &str) -> bool {
    matches!(
        identifier,
        "ObjectId"
            | "ObjectId.fromDate"
            | "ISODate"
            | "Date"
            | "NumberDecimal"
            | "NumberLong"
            | "NumberInt"
            | "NumberDouble"
            | "Number"
            | "String"
            | "Boolean"
            | "BinData"
            | "HexData"
            | "UUID"
            | "Timestamp"
            | "RegExp"
            | "Code"
            | "Array"
            | "Object"
            | "DBRef"
            | "MinKey"
            | "MaxKey"
            | "Undefined"
    )
}

fn convert_special_construct(
    chars: &[char],
    after_identifier: usize,
    identifier: &str,
) -> Result<Option<(String, usize)>, String> {
    let mut index = after_identifier;
    let len = chars.len();

    while index < len && chars[index].is_whitespace() {
        index += 1;
    }

    if index >= len || chars[index] != '(' {
        return Ok(None);
    }

    index += 1;
    let args_start = index;
    let mut depth = 0usize;

    while index < len {
        match chars[index] {
            '(' => {
                depth += 1;
                index += 1;
            }
            ')' => {
                if depth == 0 {
                    let args: String = chars[args_start..index].iter().collect();
                    let replacement = build_extended_json(identifier, &args)?;
                    return Ok(Some((replacement, index + 1)));
                }
                depth -= 1;
                index += 1;
            }
            '\'' => {
                index = skip_single_quoted(chars, index)?;
            }
            '\"' => {
                index = skip_double_quoted(chars, index)?;
            }
            _ => index += 1,
        }
    }

    Err(tr_format("Call parenthesis for {} is not closed.", &[identifier]))
}

fn build_extended_json(identifier: &str, args: &str) -> Result<String, String> {
    let parts = split_arguments(args);
    let bson = build_special_bson(identifier, &parts)?;
    bson_to_extended_json(bson)
}

fn build_special_bson(identifier: &str, parts: &[String]) -> Result<Bson, String> {
    match identifier {
        "ObjectId" => {
            let object_id = match parts.len() {
                0 => ObjectId::new(),
                1 => {
                    let value = parse_shell_json_value(&parts[0])?;
                    let hex = value_as_string(&value)?;
                    ObjectId::from_str(&hex).map_err(|_| {
                        String::from(tr(
                            "ObjectId requires a 24-character hex string or no arguments.",
                        ))
                    })?
                }
                _ => {
                    return Err(String::from(tr(
                        "ObjectId accepts either zero or one string argument.",
                    )));
                }
            };
            Ok(Bson::ObjectId(object_id))
        }
        "ObjectId.fromDate" => {
            if parts.len() != 1 {
                return Err(String::from(tr("ObjectId.fromDate expects a single argument.")));
            }
            let date = parse_date_constructor(&[parts[0].clone()])?;
            let seconds = (date.timestamp_millis() / 1000) as u32;
            Ok(Bson::ObjectId(ObjectId::from_parts(seconds, [0; 5], [0; 3])))
        }
        "ISODate" | "Date" => {
            let datetime = parse_date_constructor(parts)?;
            Ok(Bson::DateTime(datetime))
        }
        "NumberDecimal" => {
            let literal = parts.get(0).cloned().unwrap_or_else(|| String::from(tr("0")));
            let value = parse_shell_json_value(&literal)?;
            let text = value_as_string(&value)?;
            let decimal = Decimal128::from_str(&text)
                .map_err(|_| String::from(tr("NumberDecimal expects a valid decimal value.")))?;
            Ok(Bson::Decimal128(decimal))
        }
        "NumberLong" => {
            let literal = parts.get(0).cloned().unwrap_or_else(|| String::from(tr("0")));
            let value = parse_shell_json_value(&literal)?;
            let text = value_as_string(&value)?;
            let parsed = i128::from_str(&text)
                .map_err(|_| String::from(tr("NumberLong expects an integer.")))?;
            let value = i64::try_from(parsed)
                .map_err(|_| String::from(tr("NumberLong value exceeds the i64 range.")))?;
            Ok(Bson::Int64(value))
        }
        "NumberInt" => {
            let literal = parts.get(0).cloned().unwrap_or_else(|| String::from(tr("0")));
            let value = parse_shell_json_value(&literal)?;
            let text = value_as_string(&value)?;
            let parsed = i64::from_str(&text)
                .map_err(|_| String::from(tr("NumberInt expects an integer.")))?;
            let value = i32::try_from(parsed)
                .map_err(|_| String::from(tr("NumberInt value is out of the Int32 range.")))?;
            Ok(Bson::Int32(value))
        }
        "NumberDouble" | "Number" => {
            let literal = parts.get(0).cloned().unwrap_or_else(|| String::from(tr("0")));
            let value = parse_shell_json_value(&literal)?;
            let number = value_as_f64(&value)?;
            Ok(Bson::Double(number))
        }
        "Boolean" => {
            let literal = parts.get(0).cloned().unwrap_or_else(|| String::from(tr("false")));
            let value = parse_shell_json_value(&literal)?;
            let flag = value_as_bool(&value)?;
            Ok(Bson::Boolean(flag))
        }
        "String" => {
            let text = if let Some(arg) = parts.get(0) {
                let value = parse_shell_json_value(arg)?;
                value_as_string(&value)?
            } else {
                String::new()
            };
            Ok(Bson::String(text))
        }
        "UUID" => {
            let uuid = if let Some(arg) = parts.get(0) {
                let value = parse_shell_json_value(arg)?;
                let text = value_as_string(&value)?;
                Uuid::parse_str(&text).map_err(|_| {
                    String::from(tr(
                        "UUID expects a string in the format xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx.",
                    ))
                })?
            } else {
                Uuid::new_v4()
            };
            Ok(Bson::Binary(Binary {
                subtype: BinarySubtype::Uuid,
                bytes: uuid.as_bytes().to_vec(),
            }))
        }
        "BinData" => {
            if parts.len() != 2 {
                return Err(String::from(tr(
                    "BinData expects two arguments: a subtype and a base64 string.",
                )));
            }
            let subtype_value = parse_shell_json_value(&parts[0])?;
            let subtype = value_as_u8(&subtype_value)?;
            let data_value = parse_shell_json_value(&parts[1])?;
            let encoded = data_value.as_str().ok_or_else(|| {
                String::from(tr("BinData expects a base64 string as the second argument."))
            })?;
            let bytes = BASE64_STANDARD
                .decode(encoded)
                .map_err(|_| String::from(tr("Unable to decode the BinData base64 string.")))?;
            Ok(Bson::Binary(Binary { subtype: BinarySubtype::from(subtype), bytes }))
        }
        "HexData" => {
            if parts.len() != 2 {
                return Err(String::from(tr(
                    "HexData expects two arguments: a subtype and a hex string.",
                )));
            }
            let subtype_value = parse_shell_json_value(&parts[0])?;
            let subtype = value_as_u8(&subtype_value)?;
            let hex_value = parse_shell_json_value(&parts[1])?;
            let hex_string = hex_value.as_str().ok_or_else(|| {
                String::from(tr("HexData expects a string as the second argument."))
            })?;
            let bytes = decode_hex(hex_string)?;
            Ok(Bson::Binary(Binary { subtype: BinarySubtype::from(subtype), bytes }))
        }
        "Array" => {
            let mut items = Vec::new();
            for part in parts {
                let value = parse_shell_bson_value(part)?;
                items.push(value);
            }
            Ok(Bson::Array(items))
        }
        "Object" => {
            if parts.is_empty() {
                return Ok(Bson::Document(Document::new()));
            }
            let value = parse_shell_bson_value(&parts[0])?;
            match value {
                Bson::Document(doc) => Ok(Bson::Document(doc)),
                other => Err(tr_format(
                    "Object expects a JSON object, but received a value of type {}.",
                    &[&format!("{other:?}")],
                )),
            }
        }
        "Timestamp" => {
            if parts.len() != 2 {
                return Err(String::from(tr(
                    "Timestamp expects two arguments: time and increment.",
                )));
            }
            let time = parse_timestamp_seconds(&parts[0])?;
            let increment = parse_u32_argument(&parts[1], "Timestamp", "i")?;
            Ok(Bson::Timestamp(BsonTimestamp { time, increment }))
        }
        "RegExp" => {
            if parts.is_empty() || parts.len() > 2 {
                return Err(String::from(tr("RegExp expects a pattern and optional options.")));
            }
            let pattern_value = parse_shell_json_value(&parts[0])?;
            let pattern = pattern_value
                .as_str()
                .ok_or_else(|| String::from(tr("RegExp expects a string pattern.")))?
                .to_string();
            let options = if let Some(arg) = parts.get(1) {
                let options_value = parse_shell_json_value(arg)?;
                options_value
                    .as_str()
                    .ok_or_else(|| String::from(tr("RegExp options must be a string.")))?
                    .to_string()
            } else {
                String::new()
            };
            Ok(Bson::RegularExpression(Regex { pattern, options }))
        }
        "Code" => {
            let code_text = parts.get(0).cloned().unwrap_or_else(String::new);
            let code_value = parse_shell_json_value(&code_text)?;
            let code = value_as_string(&code_value)?;
            if let Some(scope_part) = parts.get(1) {
                let scope_bson = parse_shell_bson_value(scope_part)?;
                let scope = match scope_bson {
                    Bson::Document(doc) => doc,
                    _ => {
                        return Err(String::from(tr(
                            "The second argument to Code must be an object.",
                        )));
                    }
                };
                Ok(Bson::JavaScriptCodeWithScope(JavaScriptCodeWithScope { code, scope }))
            } else {
                Ok(Bson::JavaScriptCode(code))
            }
        }
        "DBRef" => {
            if parts.len() < 2 || parts.len() > 3 {
                return Err(String::from(tr(
                    "DBRef expects two or three arguments: collection, _id, and an optional database name.",
                )));
            }
            let collection_value = parse_shell_json_value(&parts[0])?;
            let collection = value_as_string(&collection_value)?;
            let id_bson = parse_shell_bson_value(&parts[1])?;
            let id = match id_bson {
                Bson::ObjectId(oid) => oid,
                _ => {
                    return Err(String::from(tr(
                        "DBRef expects an ObjectId as the second argument.",
                    )));
                }
            };
            let db_name = if let Some(db_part) = parts.get(2) {
                let value = parse_shell_json_value(db_part)?;
                Some(value_as_string(&value)?)
            } else {
                None
            };
            let mut doc = Document::new();
            doc.insert("$ref", Bson::String(collection));
            doc.insert("$id", Bson::ObjectId(id));
            if let Some(db) = db_name {
                doc.insert("$db", Bson::String(db));
            }
            Ok(Bson::Document(doc))
        }
        "MinKey" => Ok(Bson::MinKey),
        "MaxKey" => Ok(Bson::MaxKey),
        "Undefined" => Ok(Bson::Undefined),
        _ => Err(tr_format("Constructor '{}' is not supported.", &[identifier])),
    }
}

fn bson_to_extended_json(value: Bson) -> Result<String, String> {
    let extended = value.into_relaxed_extjson();
    serde_json::to_string(&extended).map_err(|error| format!("JSON serialization error: {error}"))
}

pub fn parse_shell_bson_value(source: &str) -> Result<Bson, String> {
    let normalized = preprocess_shell_json(source)?;
    serde_json::from_str(&normalized).map_err(|error| format!("JSON parse error: {error}"))
}

pub(crate) fn value_as_bool(value: &Value) -> Result<bool, String> {
    if let Some(flag) = value.as_bool() {
        Ok(flag)
    } else if let Some(number) = value.as_i64() {
        Ok(number != 0)
    } else if let Some(number) = value.as_u64() {
        Ok(number != 0)
    } else if let Some(text) = value.as_str() {
        match text.trim().to_lowercase().as_str() {
            "true" | "1" => Ok(true),
            "false" | "0" => Ok(false),
            _ => Err(String::from(tr("String must be true or false."))),
        }
    } else {
        Err(String::from(tr("Value must be boolean, numeric, or a string equal to true/false.")))
    }
}

pub(crate) fn value_as_f64(value: &Value) -> Result<f64, String> {
    if let Some(number) = value.as_f64() {
        Ok(number)
    } else if let Some(number) = value.as_i64() {
        Ok(number as f64)
    } else if let Some(number) = value.as_u64() {
        Ok(number as f64)
    } else if let Some(text) = value.as_str() {
        match text.trim().to_lowercase().as_str() {
            "infinity" => Ok(f64::INFINITY),
            "-infinity" => Ok(f64::NEG_INFINITY),
            "nan" => Ok(f64::NAN),
            other => other
                .parse::<f64>()
                .map_err(|_| String::from(tr("Failed to convert string value to number."))),
        }
    } else {
        Err(String::from(tr("Value must be a number or a string.")))
    }
}

pub(crate) fn parse_date_constructor(parts: &[String]) -> Result<DateTime, String> {
    if parts.is_empty() {
        return Ok(DateTime::now());
    }

    fn parse_iso_like(text: &str) -> Option<DateTime> {
        let trimmed = text.trim();

        if let Ok(dt) = DateTime::parse_rfc3339_str(trimmed) {
            return Some(dt);
        }

        let normalized_with_tz = if let Some(stripped) = trimmed.strip_suffix('Z') {
            format!("{stripped}+00:00")
        } else {
            trimmed.to_string()
        };

        let tz_patterns = [
            "%Y-%m-%dT%H:%M:%S%.f%:z",
            "%Y-%m-%dT%H:%M:%S%:z",
            "%Y-%m-%dT%H:%M%:z",
            "%Y-%m-%dT%H%:z",
            "%Y-%m-%d %H:%M:%S%.f%:z",
            "%Y-%m-%d %H:%M:%S%:z",
            "%Y-%m-%d %H:%M%:z",
            "%Y-%m-%d %H%:z",
        ];

        for fmt in tz_patterns {
            if let Ok(dt) = chrono::DateTime::parse_from_str(&normalized_with_tz, fmt) {
                return Some(DateTime::from_millis(dt.with_timezone(&Utc).timestamp_millis()));
            }
        }

        let naive_patterns = [
            "%Y-%m-%dT%H:%M:%S%.f",
            "%Y-%m-%dT%H:%M:%S",
            "%Y-%m-%dT%H:%M",
            "%Y-%m-%dT%H",
            "%Y-%m-%d %H:%M:%S%.f",
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d %H:%M",
            "%Y-%m-%d %H",
            "%Y-%m-%d",
        ];

        for fmt in naive_patterns {
            if let Ok(ndt) = NaiveDateTime::parse_from_str(trimmed, fmt) {
                let dt = Utc.from_utc_datetime(&ndt);
                return Some(DateTime::from_millis(dt.timestamp_millis()));
            }
        }

        if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
            if let Some(ndt) = date.and_hms_opt(0, 0, 0) {
                let dt = Utc.from_utc_datetime(&ndt);
                return Some(DateTime::from_millis(dt.timestamp_millis()));
            }
        }

        None
    }

    if parts.len() == 1 {
        let bson = parse_shell_bson_value(&parts[0])?;
        return match bson {
            Bson::DateTime(dt) => Ok(dt),
            Bson::String(text) => {
                if let Some(dt) = parse_iso_like(&text) {
                    Ok(dt)
                } else if let Ok(ms) = text.parse::<i128>() {
                    Ok(DateTime::from_millis(ms as i64))
                } else {
                    Err(String::from(tr("Failed to convert string to date.")))
                }
            }
            Bson::Int32(value) => Ok(DateTime::from_millis(value as i64)),
            Bson::Int64(value) => Ok(DateTime::from_millis(value)),
            Bson::Double(value) => Ok(DateTime::from_millis(value as i64)),
            Bson::Decimal128(value) => {
                let millis = value
                    .to_string()
                    .parse::<f64>()
                    .map_err(|_| String::from(tr("Failed to convert Decimal128 to a number.")))?;
                Ok(DateTime::from_millis(millis as i64))
            }
            Bson::Null => Ok(DateTime::now()),
            other => Err(tr_format(
                "Cannot convert value of type {} to a date.",
                &[&format!("{other:?}")],
            )),
        };
    }

    construct_date_from_components(parts)
}

fn construct_date_from_components(parts: &[String]) -> Result<DateTime, String> {
    let mut components = [0i64; 7];
    for (index, part) in parts.iter().enumerate().take(7) {
        let value = parse_shell_json_value(part)?;
        let number = value_as_f64(&value)?;
        components[index] = number.trunc() as i64;
    }

    let year = components[0] as i32;
    let month_zero = components.get(1).copied().unwrap_or(0);
    let month = (month_zero + 1).clamp(1, 12) as u32;
    let day = components.get(2).copied().unwrap_or(1).clamp(1, 31) as u32;
    let hour = components.get(3).copied().unwrap_or(0).clamp(0, 23) as u32;
    let minute = components.get(4).copied().unwrap_or(0).clamp(0, 59) as u32;
    let second = components.get(5).copied().unwrap_or(0).clamp(0, 59) as u32;
    let millis = components.get(6).copied().unwrap_or(0);

    let base =
        Utc.with_ymd_and_hms(year, month, day, hour, minute, second).single().ok_or_else(|| {
            String::from(tr("Unable to construct a date with the specified components."))
        })?;

    let chrono_dt = base + ChronoDuration::milliseconds(millis);
    Ok(DateTime::from_millis(chrono_dt.timestamp_millis()))
}

fn parse_timestamp_seconds(value: &str) -> Result<u32, String> {
    let trimmed = value.trim();
    if let Some(prefix) = trimmed.strip_suffix(".getTime()/1000") {
        let date = parse_date_constructor(&[prefix.trim().to_string()])?;
        return Ok((date.timestamp_millis() / 1000) as u32);
    }

    if let Some(prefix) = trimmed.strip_suffix(".getTime()") {
        let date = parse_date_constructor(&[prefix.trim().to_string()])?;
        return Ok(date.timestamp_millis() as u32);
    }

    let bson = parse_shell_bson_value(trimmed)?;
    match bson {
        Bson::DateTime(dt) => Ok((dt.timestamp_millis() / 1000) as u32),
        Bson::Int32(value) => Ok(value as u32),
        Bson::Int64(value) => u32::try_from(value)
            .map_err(|_| String::from(tr("Timestamp time value must fit into u32."))),
        Bson::Double(value) => Ok(value as u32),
        Bson::String(text) => {
            if let Ok(dt) = DateTime::parse_rfc3339_str(&text) {
                Ok((dt.timestamp_millis() / 1000) as u32)
            } else {
                let number = text.parse::<f64>().map_err(|_| {
                    String::from(tr("String value in Timestamp must be a number or an ISO date."))
                })?;
                Ok(number as u32)
            }
        }
        other => Err(tr_format(
            "The first argument to Timestamp must be a number or a date; received {}.",
            &[&format!("{other:?}")],
        )),
    }
}

fn parse_u32_argument(value: &str, context: &str, field: &str) -> Result<u32, String> {
    let bson = parse_shell_bson_value(value)?;
    match bson {
        Bson::Int32(v) => Ok(v as u32),
        Bson::Int64(v) => {
            u32::try_from(v).map_err(|_| tr_format("{}::{} must fit into u32.", &[context, field]))
        }
        Bson::Double(v) => Ok(v as u32),
        Bson::String(text) => text
            .parse::<u32>()
            .map_err(|_| tr_format("{}::{} must be a positive integer.", &[context, field])),
        other => Err(tr_format(
            "{}::{} must be a number, received {}.",
            &[context, field, &format!("{other:?}")],
        )),
    }
}

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    let cleaned: String = value.chars().filter(|ch| !ch.is_whitespace()).collect();
    if cleaned.len() % 2 != 0 {
        return Err(String::from(tr("Hex string must contain an even number of characters.")));
    }
    let mut bytes = Vec::with_capacity(cleaned.len() / 2);
    let chars: Vec<char> = cleaned.chars().collect();
    for chunk in chars.chunks(2) {
        let high = hex_value(chunk[0])?;
        let low = hex_value(chunk[1])?;
        bytes.push(((high << 4) | low) as u8);
    }
    Ok(bytes)
}

fn value_as_string(value: &Value) -> Result<String, String> {
    if let Some(s) = value.as_str() {
        Ok(s.to_string())
    } else if value.is_number() {
        Ok(value.to_string())
    } else {
        Err(String::from(tr("Argument must be a string or a number.")))
    }
}

fn value_as_u8(value: &Value) -> Result<u8, String> {
    if let Some(number) = value.as_u64() {
        u8::try_from(number)
            .map_err(|_| String::from(tr("BinData subtype must be a number from 0 to 255.")))
    } else if let Some(number) = value.as_i64() {
        if (0..=255).contains(&number) {
            Ok(number as u8)
        } else {
            Err(String::from(tr("BinData subtype must be a number from 0 to 255.")))
        }
    } else if let Some(text) = value.as_str() {
        u8::from_str_radix(text, 16)
            .map_err(|_| String::from(tr("BinData subtype must be a number or a hex string.")))
    } else {
        Err(String::from(tr("BinData subtype must be a number.")))
    }
}

pub fn format_shell_value(value: &Bson) -> String {
    match value {
        Bson::String(text) => text.clone(),
        _ => format_bson_shell(value),
    }
}

pub fn bson_type_name(bson: &Bson) -> &'static str {
    match bson {
        Bson::Document(_) => "Document",
        Bson::Array(_) => "Array",
        Bson::String(_) => "String",
        Bson::Boolean(_) => "Boolean",
        Bson::Int32(_) => "Int32",
        Bson::Int64(_) => "Int64",
        Bson::Double(_) => "Double",
        Bson::Decimal128(_) => "Decimal128",
        Bson::DateTime(_) => "DateTime",
        Bson::ObjectId(_) => "ObjectId",
        Bson::Binary(binary) => {
            if binary.subtype == BinarySubtype::Uuid && binary.bytes.len() == 16 {
                "UUID"
            } else {
                "Binary"
            }
        }
        Bson::RegularExpression(_) => "RegExp",
        Bson::JavaScriptCode(_) => "Code",
        Bson::JavaScriptCodeWithScope(_) => "CodeWithScope",
        Bson::Timestamp(_) => "Timestamp",
        Bson::DbPointer(_) => "DBRef",
        Bson::Undefined => "Undefined",
        Bson::Null => "Null",
        Bson::MinKey => "MinKey",
        Bson::MaxKey => "MaxKey",
        Bson::Symbol(_) => "Symbol",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, TimeZone};
    use mongodb::bson::{
        Binary, DateTime, Decimal128, JavaScriptCodeWithScope, Regex, Timestamp as BsonTimestamp,
        doc, oid::ObjectId,
    };
    use std::str::FromStr;
    use uuid::Uuid;

    // --- Tests for `split_arguments` ---

    #[test]
    fn test_split_arguments() {
        assert_eq!(split_arguments("1, 2, 3"), vec!["1", "2", "3"]);
        assert_eq!(split_arguments("\"hello\", \"world\""), vec!["\"hello\"", "\"world\""]);
        assert_eq!(split_arguments("{a: 1}, [1, 2]"), vec!["{a: 1}", "[1, 2]"]);
        assert_eq!(split_arguments("{a: 1, b: 2}, [1, 2]"), vec!["{a: 1, b: 2}", "[1, 2]"]);
        assert_eq!(split_arguments("0, \"base64string\""), vec!["0", "\"base64string\""]);
        assert_eq!(split_arguments(""), Vec::<String>::new());
        assert_eq!(split_arguments("   "), Vec::<String>::new());
        assert_eq!(split_arguments("hello"), vec!["hello"]);
        assert_eq!(split_arguments("\"a,b\", \"c,d\""), vec!["\"a,b\"", "\"c,d\""]);
        assert_eq!(
            split_arguments("function() { return 1, 2; }"),
            vec!["function() { return 1, 2; }"]
        );
        assert_eq!(
            split_arguments("Code(\"return a;\", { a: 1 })"),
            vec!["Code(\"return a;\", { a: 1 })"]
        );
    }

    #[test]
    fn test_split_arguments_complex_cases() {
        assert_eq!(
            split_arguments(
                "'text, with comma', Array(1, 2), function() { return { nested: true }; }"
            ),
            vec!["'text, with comma'", "Array(1, 2)", "function() { return { nested: true }; }",]
        );

        assert_eq!(
            split_arguments("Code('fn()', { inner: [1, 2] }), new Date(2020, 0, 1)"),
            vec!["Code('fn()', { inner: [1, 2] })", "new Date(2020, 0, 1)",]
        );
    }

    // --- Tests for `format_bson_shell` (format in string) ---

    #[test]
    fn test_format_bson_shell_scalars() {
        assert_eq!(format_bson_shell(&Bson::String("hello".to_string())), "\"hello\"");
        assert_eq!(format_bson_shell(&Bson::Int32(123)), "123");
        assert_eq!(format_bson_shell(&Bson::Int64(456)), "456");
        assert_eq!(format_bson_shell(&Bson::Boolean(true)), "true");
        assert_eq!(format_bson_shell(&Bson::Null), "null");
        assert_eq!(format_bson_shell(&Bson::Double(123.45)), "123.45");
        assert_eq!(format_bson_shell(&Bson::Double(f64::NAN)), "NaN");
        assert_eq!(format_bson_shell(&Bson::Double(f64::INFINITY)), "Infinity");
        assert_eq!(format_bson_shell(&Bson::Double(f64::NEG_INFINITY)), "-Infinity");
    }

    #[test]
    fn test_format_bson_shell_special_types() {
        let oid = ObjectId::from_str("605c7d5c5b5d7b5d7b5d7b5d").unwrap();
        assert_eq!(
            format_bson_shell(&Bson::ObjectId(oid)),
            "ObjectId(\"605c7d5c5b5d7b5d7b5d7b5d\")"
        );

        let dt = DateTime::from_millis(1616671580000); // 2021-03-25T11:26:20Z
        assert_eq!(format_bson_shell(&Bson::DateTime(dt)), "ISODate(\"2021-03-25T11:26:20Z\")");

        let dec = Decimal128::from_str("123.456").unwrap();
        assert_eq!(format_bson_shell(&Bson::Decimal128(dec)), "NumberDecimal(\"123.456\")");

        let uuid_str = "d3f4b7a0-2b7e-4b7e-8b7e-d3f4b7a02b7e";
        let uuid = Uuid::parse_str(uuid_str).unwrap();
        let bin =
            Bson::Binary(Binary { subtype: BinarySubtype::Uuid, bytes: uuid.as_bytes().to_vec() });
        assert_eq!(format_bson_shell(&bin), format!("UUID(\"{uuid_str}\")"));

        let bin_generic =
            Bson::Binary(Binary { subtype: BinarySubtype::Generic, bytes: vec![0, 1, 2, 3] });
        assert_eq!(format_bson_shell(&bin_generic), "BinData(0, \"AAECAw==\")");

        let regex =
            Bson::RegularExpression(Regex { pattern: "foo".to_string(), options: "i".to_string() });
        assert_eq!(format_bson_shell(&regex), "RegExp(\"foo\", \"i\")");

        let ts = Bson::Timestamp(BsonTimestamp { time: 12345, increment: 67890 });
        assert_eq!(format_bson_shell(&ts), "Timestamp(12345, 67890)");

        assert_eq!(format_bson_shell(&Bson::MinKey), "MinKey()");
        assert_eq!(format_bson_shell(&Bson::MaxKey), "MaxKey()");
        assert_eq!(format_bson_shell(&Bson::Undefined), "undefined");
    }

    #[test]
    fn test_format_bson_shell_containers() {
        let doc = doc! { "a": 1, "b": "hello" };
        let expected_doc = "{\n    \"a\": 1,\n    \"b\": \"hello\"\n}";
        assert_eq!(format_bson_shell(&Bson::Document(doc)), expected_doc);

        let arr = Bson::Array(vec![Bson::Int32(1), Bson::String("foo".to_string())]);
        let expected_arr = "[\n    1,\n    \"foo\"\n]";
        assert_eq!(format_bson_shell(&arr), expected_arr);

        let nested_doc = doc! {
            "a": 1,
            "nested": {
                "b": true,
                "c": [1, 2]
            }
        };
        // Примечание: форматирование с отступами чувствительно к пробелам
        let expected_nested = "{\n    \"a\": 1,\n    \"nested\": {\n        \"b\": true,\n        \"c\": [\n            1,\n            2\n        ]\n    }\n}";
        assert_eq!(format_bson_shell(&Bson::Document(nested_doc)), expected_nested);

        assert_eq!(format_bson_shell(&Bson::Document(doc! {})), "{}");
        assert_eq!(format_bson_shell(&Bson::Array(vec![])), "[]");
    }

    // --- Tests for `parse_shell_bson_value` (Парсинг из строки) ---

    #[test]
    fn test_parse_shell_bson_primitives() {
        assert_eq!(parse_shell_bson_value("123").unwrap(), Bson::Int32(123));
        assert_eq!(
            parse_shell_bson_value("9223372036854775807").unwrap(),
            Bson::Int64(9223372036854775807)
        );
        assert_eq!(parse_shell_bson_value("123.45").unwrap(), Bson::Double(123.45));
        assert_eq!(parse_shell_bson_value("\"hello\"").unwrap(), Bson::String("hello".to_string()));
        assert_eq!(parse_shell_bson_value("'hello'").unwrap(), Bson::String("hello".to_string()));
        assert_eq!(
            parse_shell_bson_value("'hello \\' world'").unwrap(),
            Bson::String("hello ' world".to_string())
        );
        assert_eq!(parse_shell_bson_value("'\\x41'").unwrap(), Bson::String("A".to_string()));
        assert_eq!(parse_shell_bson_value("true").unwrap(), Bson::Boolean(true));
        assert_eq!(parse_shell_bson_value("null").unwrap(), Bson::Null);
        assert_eq!(parse_shell_bson_value("undefined").unwrap(), Bson::Undefined);
        assert_eq!(parse_shell_bson_value("Infinity").unwrap(), Bson::Double(f64::INFINITY));
        assert_eq!(parse_shell_bson_value("-Infinity").unwrap(), Bson::Double(f64::NEG_INFINITY));
        // f64::NAN != f64::NAN, поэтому проверяем через is_nan()
        assert!(parse_shell_bson_value("NaN").unwrap().as_f64().unwrap().is_nan());
    }

    #[test]
    fn test_parse_shell_bson_regex_literal() {
        let expected =
            Bson::RegularExpression(Regex { pattern: "foo".to_string(), options: "i".to_string() });
        assert_eq!(parse_shell_bson_value("/foo/i").unwrap(), expected);

        let expected_no_opts =
            Bson::RegularExpression(Regex { pattern: "bar".to_string(), options: "".to_string() });
        assert_eq!(parse_shell_bson_value("/bar/").unwrap(), expected_no_opts);

        // Парсер должен падать на неоднозначных выражениях, т.к. он не полный JS-парсер
        assert!(parse_shell_bson_value("1 / 2").is_err(), "Должен упасть парсинг '1 / 2'");

        // Проверка корректного парсинга в контексте
        assert_eq!(
            parse_shell_bson_value("{ \"a\": /foo/i }").unwrap(),
            // Вот исправление: оборачиваем doc! в Bson::Document()
            Bson::Document(
                doc! { "a": Bson::RegularExpression(Regex { pattern: "foo".to_string(), options: "i".to_string() }) }
            )
        );
        assert_eq!(
            parse_shell_bson_value("[ /foo/i ]").unwrap(),
            Bson::Array(vec![Bson::RegularExpression(Regex {
                pattern: "foo".to_string(),
                options: "i".to_string()
            })])
        );
    }

    #[test]
    fn test_parse_shell_bson_function_literal() {
        let code = "function(a, b) { return a + b; }";
        let expected = Bson::JavaScriptCode(code.to_string());
        assert_eq!(parse_shell_bson_value(code).unwrap(), expected);

        let complex_code = "function () { var x = 'a{}b'; return x; }";
        let expected_complex = Bson::JavaScriptCode(complex_code.to_string());
        assert_eq!(parse_shell_bson_value(complex_code).unwrap(), expected_complex);
    }

    #[test]
    fn test_parse_shell_bson_constructors() {
        let oid_str = "605c7d5c5b5d7b5d7b5d7b5d";
        let oid = ObjectId::from_str(oid_str).unwrap();
        assert_eq!(
            parse_shell_bson_value(&format!("ObjectId(\"{oid_str}\")")).unwrap(),
            Bson::ObjectId(oid)
        );
        // Проверка с 'new'
        assert_eq!(
            parse_shell_bson_value(&format!("new ObjectId(\"{oid_str}\")")).unwrap(),
            Bson::ObjectId(oid)
        );

        // Checking ISODate in full ISO 8601 format and date-only format
        let dt_str = "2021-03-25T11:26:20Z";
        let dt = DateTime::from_millis(1616671580000);
        assert_eq!(
            parse_shell_bson_value(&format!("ISODate(\"{dt_str}\")")).unwrap(),
            Bson::DateTime(dt)
        );

        let dt_str = "2025-12-01";
        let dt = DateTime::from_millis(1764547200000);
        assert_eq!(
            parse_shell_bson_value(&format!("ISODate(\"{dt_str}\")")).unwrap(),
            Bson::DateTime(dt)
        );
        assert_eq!(parse_shell_bson_value("Date(1764547200000)").unwrap(), Bson::DateTime(dt));

        // Проверка Date(y, m, d) (в JS месяцы 0-индексированные)
        let dt_comp = Utc.with_ymd_and_hms(2021, 3, 25, 0, 0, 0).single().unwrap();
        assert_eq!(
            parse_shell_bson_value("Date(2021, 2, 25)").unwrap(),
            Bson::DateTime(DateTime::from_millis(dt_comp.timestamp_millis()))
        );

        let dec = Decimal128::from_str("123.456").unwrap();
        assert_eq!(
            parse_shell_bson_value("NumberDecimal(\"123.456\")").unwrap(),
            Bson::Decimal128(dec)
        );
        // Парсер также должен обрабатывать числа
        assert_eq!(
            parse_shell_bson_value("NumberDecimal(123.456)").unwrap(),
            Bson::Decimal128(dec)
        );

        let long_val = 9223372036854775807i64;
        assert_eq!(
            parse_shell_bson_value(&format!("NumberLong(\"{long_val}\")")).unwrap(),
            Bson::Int64(long_val)
        );
        assert_eq!(
            parse_shell_bson_value("NumberLong(9223372036854775807)").unwrap(),
            Bson::Int64(long_val)
        );
        assert_eq!(parse_shell_bson_value("NumberInt(\"123\")").unwrap(), Bson::Int32(123));

        let uuid_str = "d3f4b7a0-2b7e-4b7e-8b7e-d3f4b7a02b7e";
        let uuid = Uuid::parse_str(uuid_str).unwrap();
        let uuid_bin =
            Bson::Binary(Binary { subtype: BinarySubtype::Uuid, bytes: uuid.as_bytes().to_vec() });
        assert_eq!(parse_shell_bson_value(&format!("UUID(\"{uuid_str}\")")).unwrap(), uuid_bin);

        let bin_generic =
            Bson::Binary(Binary { subtype: BinarySubtype::Generic, bytes: vec![0, 1, 2, 3] });
        assert_eq!(parse_shell_bson_value("BinData(0, \"AAECAw==\")").unwrap(), bin_generic);
        assert_eq!(parse_shell_bson_value("HexData(0, \"00010203\")").unwrap(), bin_generic);
        // Проверка HexData с пробелами
        assert_eq!(parse_shell_bson_value("HexData(0, \"00 01 02 03\")").unwrap(), bin_generic);

        let ts = Bson::Timestamp(BsonTimestamp { time: 12345, increment: 67890 });
        assert_eq!(parse_shell_bson_value("Timestamp(12345, 67890)").unwrap(), ts);

        let regex =
            Bson::RegularExpression(Regex { pattern: "foo".to_string(), options: "i".to_string() });
        assert_eq!(parse_shell_bson_value("RegExp(\"foo\", \"i\")").unwrap(), regex);

        let code = Bson::JavaScriptCode("function() {}".to_string());
        assert_eq!(parse_shell_bson_value("Code(\"function() {}\")").unwrap(), code);

        let code_ws = Bson::JavaScriptCodeWithScope(JavaScriptCodeWithScope {
            code: "return a;".to_string(),
            scope: doc! { "a": 1 },
        });
        assert_eq!(parse_shell_bson_value("Code(\"return a;\", { \"a\": 1 })").unwrap(), code_ws);

        assert_eq!(parse_shell_bson_value("MinKey()").unwrap(), Bson::MinKey);
        assert_eq!(parse_shell_bson_value("MaxKey()").unwrap(), Bson::MaxKey);
    }

    #[test]
    fn test_parse_shell_bson_additional_constructors() {
        let dt_source = "2024-01-01T00:00:00Z";
        let dt = DateTime::parse_rfc3339_str(dt_source).unwrap();
        match parse_shell_bson_value(&format!("ObjectId.fromDate(ISODate(\"{dt_source}\"))"))
            .unwrap()
        {
            Bson::ObjectId(oid) => assert_eq!(oid.timestamp(), dt),
            other => panic!("expected ObjectId, got {:?}", other),
        }

        assert_eq!(parse_shell_bson_value("Number(\"42.5\")").unwrap(), Bson::Double(42.5));

        assert_eq!(parse_shell_bson_value("Boolean('true')").unwrap(), Bson::Boolean(true));

        assert_eq!(
            parse_shell_bson_value("Array(1, 2, 3)").unwrap(),
            Bson::Array(vec![Bson::Int32(1), Bson::Int32(2), Bson::Int32(3)])
        );

        assert_eq!(
            parse_shell_bson_value("Object({ \"a\": 1 })").unwrap(),
            Bson::Document(doc! { "a": 1 })
        );
    }

    #[test]
    fn test_parse_shell_bson_constructor_errors() {
        assert!(parse_shell_bson_value("HexData(0, \"001\")").is_err());
        assert!(parse_shell_bson_value("BinData(300, \"AA==\")").is_err());
        assert!(parse_shell_bson_value("Timestamp(\"foo\", 1)").is_err());
        assert!(parse_shell_bson_value("Boolean(\"not bool\")").is_err());
        assert!(parse_shell_bson_value("DBRef(\"coll\")").is_err());
        assert!(parse_shell_bson_value("Object('not object')").is_err());
    }

    // --- Tests for `parse_shell_document` и `parse_shell_array` ---

    #[test]
    fn test_parse_shell_document_and_array() {
        let doc_str = "{ \"a\": 1, \"b\": \"hello\", \"c\": ISODate(\"2021-03-25T11:26:20Z\") }";
        let dt = DateTime::from_millis(1616671580000);
        let expected_doc = doc! {
            "a": 1,
            "b": "hello",
            "c": Bson::DateTime(dt)
        };
        assert_eq!(parse_shell_document(doc_str).unwrap(), Bson::Document(expected_doc));

        // Должен упасть, если это не документ
        assert!(parse_shell_document("[1, 2]").is_err());

        let arr_str = "[1, \"hello\", /foo/i, { \"a\": MinKey() }]";
        let expected_arr = Bson::Array(vec![
            Bson::Int32(1),
            Bson::String("hello".to_string()),
            Bson::RegularExpression(Regex { pattern: "foo".to_string(), options: "i".to_string() }),
            Bson::Document(doc! { "a": Bson::MinKey }),
        ]);
        assert_eq!(parse_shell_array(arr_str).unwrap(), expected_arr);

        // Должен упасть, если это не массив
        assert!(parse_shell_array("{ \"a\": 1 }").is_err());
    }

    // --- Tests for утилит ---

    #[test]
    fn test_bson_type_name() {
        assert_eq!(bson_type_name(&Bson::Int32(1)), "Int32");
        assert_eq!(bson_type_name(&Bson::String("s".to_string())), "String");
        assert_eq!(bson_type_name(&Bson::Document(doc! {})), "Document");
        assert_eq!(bson_type_name(&Bson::Array(vec![])), "Array");
        assert_eq!(bson_type_name(&Bson::ObjectId(ObjectId::new())), "ObjectId");
        assert_eq!(bson_type_name(&Bson::Null), "Null");

        let uuid = Uuid::new_v4();
        let uuid_bin =
            Bson::Binary(Binary { subtype: BinarySubtype::Uuid, bytes: uuid.as_bytes().to_vec() });
        assert_eq!(bson_type_name(&uuid_bin), "UUID");

        let generic_bin = Bson::Binary(Binary { subtype: BinarySubtype::Generic, bytes: vec![1] });
        assert_eq!(bson_type_name(&generic_bin), "Binary");
    }

    #[test]
    fn test_parse_date_constructor_iso_variants() {
        let base_iso =
            parse_date_constructor(&[String::from("\"2025-12-01\"")]).expect("plain ISO date");
        let expected_base = Utc.with_ymd_and_hms(2025, 12, 1, 0, 0, 0).single().unwrap();
        assert_eq!(base_iso, DateTime::from_millis(expected_base.timestamp_millis()));

        let iso_with_time = parse_date_constructor(&[String::from("\"2025-12-01T15:30\"")])
            .expect("ISO date with hours and minutes");
        let expected_time = Utc.with_ymd_and_hms(2025, 12, 1, 15, 30, 0).single().unwrap();
        assert_eq!(iso_with_time, DateTime::from_millis(expected_time.timestamp_millis()));

        let iso_with_seconds_tz =
            parse_date_constructor(&[String::from("\"2025-12-01T15:30:45+03:00\"")])
                .expect("ISO date with timezone offset");
        let offset = FixedOffset::east_opt(3 * 3600).unwrap();
        let expected_tz =
            offset.with_ymd_and_hms(2025, 12, 1, 15, 30, 45).single().unwrap().with_timezone(&Utc);
        assert_eq!(iso_with_seconds_tz, DateTime::from_millis(expected_tz.timestamp_millis()));

        let iso_with_millis_z =
            parse_date_constructor(&[String::from("\"2025-12-01T00:00:00.123Z\"")])
                .expect("ISO date with milliseconds and Z");
        let expected_z = Utc.with_ymd_and_hms(2025, 12, 1, 0, 0, 0).single().unwrap()
            + chrono::Duration::milliseconds(123);
        assert_eq!(iso_with_millis_z, DateTime::from_millis(expected_z.timestamp_millis()));
    }

    #[test]
    fn test_format_bson_scalar() {
        // Эти тесты предполагают, что ваша функция `tr` возвращает
        // входную строку (например, tr("String") -> "String").
        // Если у вас настроены переводы, эти тесты могут упасть,
        // и вам нужно будет поправить ожидаемые строки.
        assert_eq!(
            format_bson_scalar(&Bson::String("hello".to_string())),
            ("hello".to_string(), "String".to_string())
        );
        assert_eq!(format_bson_scalar(&Bson::Int32(123)), ("123".to_string(), "Int32".to_string()));
        assert_eq!(
            format_bson_scalar(&Bson::Boolean(true)),
            ("true".to_string(), "Boolean".to_string())
        );
        assert_eq!(format_bson_scalar(&Bson::Null), ("null".to_string(), "Null".to_string()));

        let oid = ObjectId::from_str("605c7d5c5b5d7b5d7b5d7b5d").unwrap();
        assert_eq!(
            format_bson_scalar(&Bson::ObjectId(oid)),
            ("ObjectId(\"605c7d5c5b5d7b5d7b5d7b5d\")".to_string(), "ObjectId".to_string())
        );

        let dt = DateTime::from_millis(1616671580000); // 2021-03-25T11:26:20Z
        assert_eq!(
            format_bson_scalar(&Bson::DateTime(dt)),
            ("2021-03-25T11:26:20Z".to_string(), "DateTime".to_string())
        );

        let bin = Bson::Binary(Binary { subtype: BinarySubtype::Generic, bytes: vec![0, 1, 2] });
        assert_eq!(
            format_bson_scalar(&bin),
            ("Binary(len=3, subtype=Generic)".to_string(), "Binary".to_string())
        );
    }
}
