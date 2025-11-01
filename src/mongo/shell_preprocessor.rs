//! Utilities for normalizing Mongo shell style snippets before JSON parsing.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScanState {
    Normal,
    SingleString,
    DoubleString,
    Regex,
}

#[derive(Debug, Clone)]
struct PendingKey {
    key: String,
    whitespace: String,
    prev_non_ws: Option<char>,
}

/// Wrap unquoted object keys (e.g. `_id`, `$or`) with double quotes so the input becomes
/// valid JSON while leaving strings, regex literals and already quoted keys untouched.
pub fn quote_unquoted_keys(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut state = ScanState::Normal;
    let mut string_escape = false;
    let mut regex_escape = false;
    let mut pending: Option<PendingKey> = None;
    let mut prev_non_ws: Option<char> = None;

    for ch in input.chars() {
        match state {
            ScanState::SingleString => {
                output.push(ch);
                if string_escape {
                    string_escape = false;
                } else if ch == '\\' {
                    string_escape = true;
                } else if ch == '\'' {
                    state = ScanState::Normal;
                    prev_non_ws = Some('\'');
                }
            }
            ScanState::DoubleString => {
                output.push(ch);
                if string_escape {
                    string_escape = false;
                } else if ch == '\\' {
                    string_escape = true;
                } else if ch == '"' {
                    state = ScanState::Normal;
                    prev_non_ws = Some('"');
                }
            }
            ScanState::Regex => {
                output.push(ch);
                if regex_escape {
                    regex_escape = false;
                } else if ch == '\\' {
                    regex_escape = true;
                } else if ch == '/' {
                    state = ScanState::Normal;
                    prev_non_ws = Some('/');
                }
            }
            ScanState::Normal => {
                if let Some(mut candidate) = pending.take() {
                    match ch {
                        c if c.is_whitespace() => {
                            candidate.whitespace.push(c);
                            pending = Some(candidate);
                            continue;
                        }
                        ':' => {
                            flush_key(&mut output, &candidate, true);
                            output.push(':');
                            prev_non_ws = Some(':');
                            continue;
                        }
                        c if is_key_char(c) => {
                            candidate.key.push(c);
                            pending = Some(candidate);
                            continue;
                        }
                        _ => {
                            if let Some(last) = flush_key(&mut output, &candidate, false) {
                                prev_non_ws = Some(last);
                            }
                            // fall through to process current character
                        }
                    }
                }

                match ch {
                    c if c.is_whitespace() => {
                        output.push(c);
                    }
                    '\'' => {
                        output.push(ch);
                        state = ScanState::SingleString;
                        string_escape = false;
                    }
                    '"' => {
                        output.push(ch);
                        state = ScanState::DoubleString;
                        string_escape = false;
                    }
                    '/' => {
                        if can_start_regex(prev_non_ws) {
                            state = ScanState::Regex;
                            regex_escape = false;
                        }
                        output.push('/');
                        prev_non_ws = Some('/');
                    }
                    c if is_key_start_char(c) && should_start_key(prev_non_ws) => {
                        let mut candidate = PendingKey {
                            key: String::new(),
                            whitespace: String::new(),
                            prev_non_ws,
                        };
                        candidate.key.push(c);
                        pending = Some(candidate);
                    }
                    ':' => {
                        output.push(':');
                        prev_non_ws = Some(':');
                    }
                    _ => {
                        output.push(ch);
                        if !ch.is_whitespace() {
                            prev_non_ws = Some(ch);
                        }
                    }
                }
            }
        }
    }

    if let Some(candidate) = pending.take() {
        flush_key(&mut output, &candidate, false);
    }

    output
}

fn flush_key(output: &mut String, candidate: &PendingKey, allow_quoting: bool) -> Option<char> {
    if allow_quoting && should_quote(candidate.prev_non_ws) && !candidate.key.is_empty() {
        output.push('"');
        output.push_str(&candidate.key);
        output.push('"');
        output.push_str(&candidate.whitespace);
        Some('"')
    } else {
        output.push_str(&candidate.key);
        let last = candidate.key.chars().rev().next();
        output.push_str(&candidate.whitespace);
        last
    }
}

fn can_start_regex(prev: Option<char>) -> bool {
    match prev {
        None => true,
        Some(ch) => matches!(
            ch,
            '(' | '{'
                | '['
                | ','
                | '='
                | ':'
                | ';'
                | '!'
                | '&'
                | '|'
                | '?'
                | '+'
                | '-'
                | '*'
                | '%'
                | '^'
                | '~'
                | '<'
                | '>'
        ),
    }
}

fn is_key_start_char(ch: char) -> bool {
    ch.is_ascii_alphabetic() || matches!(ch, '_' | '$')
}

fn is_key_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$' | '.')
}

fn should_start_key(prev: Option<char>) -> bool {
    match prev {
        None => true,
        Some(ch) => matches!(ch, '{' | '[' | ',' | '('),
    }
}

fn should_quote(prev: Option<char>) -> bool {
    match prev {
        None => true,
        Some(ch) => matches!(ch, '{' | '[' | ',' | '('),
    }
}

#[cfg(test)]
mod tests {
    use super::quote_unquoted_keys;

    #[test]
    fn quotes_simple_key() {
        let input = "{_id: ObjectId('abcd1234abcd1234abcd1234')}";
        let normalized = quote_unquoted_keys(input);
        assert_eq!(normalized, "{\"_id\": ObjectId('abcd1234abcd1234abcd1234')}");
    }

    #[test]
    fn preserves_existing_quotes() {
        let input = "{\"status\": \"A\"}";
        let normalized = quote_unquoted_keys(input);
        assert_eq!(normalized, input);
    }

    #[test]
    fn handles_nested_objects() {
        let input = "{address: {city: 'NYC', zip: 10001}}";
        let normalized = quote_unquoted_keys(input);
        assert_eq!(normalized, "{\"address\": {\"city\": 'NYC', \"zip\": 10001}}");
    }

    #[test]
    fn keeps_regex_literals() {
        let input = r"{pattern: /^foo:bar$/}";
        let normalized = quote_unquoted_keys(input);
        assert_eq!(normalized, "{\"pattern\": /^foo:bar$/}");
    }

    #[test]
    fn supports_dollar_prefixed_keys() {
        let input = "{$or: [{status: 'A'}, {qty: {$lt: 30}}]}";
        let normalized = quote_unquoted_keys(input);
        assert_eq!(normalized, "{\"$or\": [{\"status\": 'A'}, {\"qty\": {\"$lt\": 30}}]}");
    }

    #[test]
    fn handles_single_quoted_keys() {
        let input = "{status:'A', 'already': 'quoted'}";
        let normalized = quote_unquoted_keys(input);
        assert_eq!(normalized, "{\"status\":'A', 'already': 'quoted'}");
    }

    #[test]
    fn preserves_whitespace_around_colon() {
        let input = "{  _id  :  1  }";
        let normalized = quote_unquoted_keys(input);
        assert_eq!(normalized, "{  \"_id\"  :  1  }");
    }

    #[test]
    fn works_with_arrays_of_objects() {
        let input = "[{a:1}, {b:2}]";
        let normalized = quote_unquoted_keys(input);
        assert_eq!(normalized, "[{\"a\":1}, {\"b\":2}]");
    }

    #[test]
    fn supports_multiline_input() {
        let input =
            "{\n  user: {\n    name: 'Bob',\n    roles: ['admin', {type: 'editor'}]\n  }\n}";
        let normalized = quote_unquoted_keys(input);
        let expected = "{\n  \"user\": {\n    \"name\": 'Bob',\n    \"roles\": ['admin', {\"type\": 'editor'}]\n  }\n}";
        assert_eq!(normalized, expected);
    }

    #[test]
    fn skips_non_object_colon_usage() {
        let input = "value ? yes : no";
        let normalized = quote_unquoted_keys(input);
        assert_eq!(normalized, input);
    }

    #[test]
    fn test_simple_unquoted() {
        let input = r#"{ key: "value" }"#;
        let expected = r#"{ "key": "value" }"#;
        assert_eq!(quote_unquoted_keys(input), expected);
    }

    #[test]
    fn test_multiple_unquoted() {
        let input = r#"{ key1: "value1", key2: 123, key3: true }"#;
        let expected = r#"{ "key1": "value1", "key2": 123, "key3": true }"#;
        assert_eq!(quote_unquoted_keys(input), expected);
    }

    #[test]
    fn test_already_double_quoted() {
        let input = r#"{ "key": "value" }"#;
        let expected = r#"{ "key": "value" }"#;
        assert_eq!(quote_unquoted_keys(input), expected);
    }

    #[test]
    fn test_already_single_quoted() {
        let input = r#"{ 'key': "value" }"#;
        let expected = r#"{ 'key': "value" }"#;
        assert_eq!(quote_unquoted_keys(input), expected);
    }

    #[test]
    fn test_mixed_quoted_and_unquoted() {
        let input = r#"{ key1: 1, "key2": 2, 'key3': 3, key4: 4 }"#;
        let expected = r#"{ "key1": 1, "key2": 2, 'key3': 3, "key4": 4 }"#;
        assert_eq!(quote_unquoted_keys(input), expected);
    }

    #[test]
    fn test_nested_object() {
        let input = r#"{ data: { payload: 123 }, user_id: "abc" }"#;
        let expected = r#"{ "data": { "payload": 123 }, "user_id": "abc" }"#;
        assert_eq!(quote_unquoted_keys(input), expected);
    }

    #[test]
    fn test_array_of_objects() {
        let input = r#"{ items: [ { id: 1 }, { id: 2 } ] }"#;
        let expected = r#"{ "items": [ { "id": 1 }, { "id": 2 } ] }"#;
        assert_eq!(quote_unquoted_keys(input), expected);
    }

    #[test]
    fn test_whitespace_variations() {
        let input = "{\n  key  :  {\n\t innerKey: \"value\"\n} , \n another: 1\n}";
        let expected = "{\n  \"key\"  :  {\n\t \"innerKey\": \"value\"\n} , \n \"another\": 1\n}";
        assert_eq!(quote_unquoted_keys(input), expected);
    }

    #[test]
    fn test_empty_object() {
        let input = r#"{}"#;
        let expected = r#"{}"#;
        assert_eq!(quote_unquoted_keys(input), expected);
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        let expected = "";
        assert_eq!(quote_unquoted_keys(input), expected);
    }

    #[test]
    fn test_mongo_style_keys() {
        let input = r#"{ _id: 1, $set: { "a.b": 1 }, $or: [ { c.d: 2 } ] }"#;
        let expected = r#"{ "_id": 1, "$set": { "a.b": 1 }, "$or": [ { "c.d": 2 } ] }"#;
        assert_eq!(quote_unquoted_keys(input), expected);
    }

    #[test]
    fn test_double_string_value_with_colon() {
        let input = r#"{ key: "a:b" }"#;
        let expected = r#"{ "key": "a:b" }"#;
        assert_eq!(quote_unquoted_keys(input), expected);
    }

    #[test]
    fn test_single_string_value_with_colon() {
        let input = r#"{ key: 'a:b' }"#;
        let expected = r#"{ "key": 'a:b' }"#;
        assert_eq!(quote_unquoted_keys(input), expected);
    }

    #[test]
    fn test_string_with_escaped_quotes() {
        let input = r#"{ key: "a \"b:c\" d", key2: 'x \'y:z\' w' }"#;
        let expected = r#"{ "key": "a \"b:c\" d", "key2": 'x \'y:z\' w' }"#;
        assert_eq!(quote_unquoted_keys(input), expected);
    }

    #[test]
    fn test_simple_regex_value() {
        let input = r#"{ filter: /abc/i }"#;
        let expected = r#"{ "filter": /abc/i }"#;
        assert_eq!(quote_unquoted_keys(input), expected);
    }

    #[test]
    fn test_regex_in_array() {
        let input = r#"[ /abc/, /def/ ]"#;
        let expected = r#"[ /abc/, /def/ ]"#;
        assert_eq!(quote_unquoted_keys(input), expected);
    }

    #[test]
    fn test_regex_with_escaped_slash() {
        let input = r#"{ regex: /a\/b:c/ }"#;
        let expected = r#"{ "regex": /a\/b:c/ }"#;
        assert_eq!(quote_unquoted_keys(input), expected);
    }

    #[test]
    fn test_division_vs_regex() {
        let input = r#"{ val: 1 / 2, regex: /a/ }"#;
        let expected = r#"{ "val": 1 / 2, "regex": /a/ }"#;
        assert_eq!(quote_unquoted_keys(input), expected);
    }

    #[test]
    fn test_string_containing_slash() {
        let input = r#"{ path: "/usr/bin", regex: /a/ }"#;
        let expected = r#"{ "path": "/usr/bin", "regex": /a/ }"#;
        assert_eq!(quote_unquoted_keys(input), expected);
    }

    #[test]
    fn test_key_is_not_quoted_if_invalid_chars() {
        let input = r#"{ a-b: 1 }"#;
        let expected = r#"{ a-b: 1 }"#;
        assert_eq!(quote_unquoted_keys(input), expected);
    }

    #[test]
    fn test_key_is_not_quoted_if_not_preceded_by_valid_char() {
        let input = r#"{ 1 key: 1 }"#;
        let expected = r#"{ 1 key: 1 }"#;
        assert_eq!(quote_unquoted_keys(input), expected);
    }

    #[test]
    fn test_complex_mongo_query() {
        let input = r#"{
            _id: { $in: [1, 2, 3] },
            status: "A",
            $or: [
                { 'qty.a': { $lt: 20 } },
                { "price.b": 10, other: /regex/ }
            ]
        }"#;
        let expected = r#"{
            "_id": { "$in": [1, 2, 3] },
            "status": "A",
            "$or": [
                { 'qty.a': { "$lt": 20 } },
                { "price.b": 10, "other": /regex/ }
            ]
        }"#;
        assert_eq!(quote_unquoted_keys(input), expected);
    }
}
