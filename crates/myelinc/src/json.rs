//! Standalone zero-dependency JSON parser and serializer for Myelin.

use std::collections::HashMap;
use crate::Value;

pub fn parse(input: &str) -> Result<Value, String> {
    let mut chars = input.char_indices().peekable();
    skip_whitespace(&mut chars);
    let val = parse_value(&mut chars)?;
    skip_whitespace(&mut chars);
    if chars.peek().is_some() {
        return Err("unexpected trailing characters after JSON value".into());
    }
    Ok(val)
}

pub fn stringify(val: &Value) -> String {
    match val {
        Value::Unit => "null".to_string(),
        Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
        Value::Int(i) => i.to_string(),
        Value::Float(f) => {
            if f.is_nan() || f.is_infinite() {
                "null".to_string()
            } else {
                f.to_string()
            }
        }
        Value::Str(s) => escape_str(s),
        Value::List(items) => {
            let rendered: Vec<String> = items.iter().map(stringify).collect();
            format!("[{}]", rendered.join(","))
        }
        Value::Map(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut pairs = Vec::new();
            for k in keys {
                let v = &map[k];
                pairs.push(format!("{}:{}", escape_str(k), stringify(v)));
            }
            format!("{{{}}}", pairs.join(","))
        }
    }
}

pub fn get_path(val: &Value, path: &str) -> Option<Value> {
    if path.is_empty() {
        return Some(val.clone());
    }
    let parts: Vec<&str> = path.split('.').collect();
    let mut cur = val;

    for part in parts {
        match cur {
            Value::Map(m) => {
                cur = m.get(part)?;
            }
            Value::List(list) => {
                let idx: usize = part.parse().ok()?;
                cur = list.get(idx)?;
            }
            _ => return None,
        }
    }
    Some(cur.clone())
}

pub fn set_path(val: &mut Value, path: &str, new_val: Value) -> Result<(), String> {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() {
        *val = new_val;
        return Ok(());
    }

    let mut cur = val;
    for (i, part) in parts.iter().enumerate() {
        let is_last = i + 1 == parts.len();
        if is_last {
            match cur {
                Value::Map(m) => {
                    m.insert((*part).to_string(), new_val);
                    return Ok(());
                }
                Value::List(l) => {
                    let idx: usize = part.parse().map_err(|_| "list index must be an integer")?;
                    if idx < l.len() {
                        l[idx] = new_val;
                        return Ok(());
                    } else {
                        return Err(format!("index {idx} out of range (len {})", l.len()));
                    }
                }
                _ => return Err("cannot set property on non-container value".into()),
            }
        } else {
            match cur {
                Value::Map(m) => {
                    if !m.contains_key(*part) {
                        m.insert((*part).to_string(), Value::Map(HashMap::new()));
                    }
                    cur = m.get_mut(*part).unwrap();
                }
                Value::List(l) => {
                    let idx: usize = part.parse().map_err(|_| "list index must be an integer")?;
                    if idx < l.len() {
                        cur = &mut l[idx];
                    } else {
                        return Err(format!("index {idx} out of range (len {})", l.len()));
                    }
                }
                _ => return Err("cannot traverse into non-container value".into()),
            }
        }
    }
    Ok(())
}

fn escape_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

type CharIter<'a> = std::iter::Peekable<std::str::CharIndices<'a>>;

fn skip_whitespace(chars: &mut CharIter) {
    while let Some(&(_, c)) = chars.peek() {
        if c.is_ascii_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}

fn parse_value(chars: &mut CharIter) -> Result<Value, String> {
    skip_whitespace(chars);
    let (_, c) = chars.peek().cloned().ok_or_else(|| "unexpected EOF".to_string())?;

    match c {
        '"' => parse_string(chars).map(Value::Str),
        '{' => parse_object(chars),
        '[' => parse_array(chars),
        't' | 'f' => parse_bool(chars),
        'n' => parse_null(chars),
        '-' | '0'..='9' => parse_number(chars),
        _ => Err(format!("unexpected character '{c}'")),
    }
}

fn parse_string(chars: &mut CharIter) -> Result<String, String> {
    chars.next(); // consume opening quote '"'
    let mut s = String::new();

    while let Some((_, c)) = chars.next() {
        match c {
            '"' => return Ok(s),
            '\\' => {
                let (_, esc) = chars.next().ok_or_else(|| "unexpected EOF in string escape".to_string())?;
                match esc {
                    '"' => s.push('"'),
                    '\\' => s.push('\\'),
                    '/' => s.push('/'),
                    'b' => s.push('\u{08}'),
                    'f' => s.push('\u{0c}'),
                    'n' => s.push('\n'),
                    'r' => s.push('\r'),
                    't' => s.push('\t'),
                    'u' => {
                        let mut hex = String::new();
                        for _ in 0..4 {
                            let (_, h) = chars.next().ok_or_else(|| "invalid unicode escape".to_string())?;
                            hex.push(h);
                        }
                        let code = u32::from_str_radix(&hex, 16)
                            .map_err(|_| format!("invalid hex in unicode escape: {hex}"))?;
                        let ch = char::from_u32(code)
                            .ok_or_else(|| format!("invalid unicode code point: {code}"))?;
                        s.push(ch);
                    }
                    _ => return Err(format!("unknown string escape '\\{esc}'")),
                }
            }
            other => s.push(other),
        }
    }
    Err("unterminated string literal".into())
}

fn parse_number(chars: &mut CharIter) -> Result<Value, String> {
    let mut num_str = String::new();
    let mut is_float = false;

    while let Some(&(_, c)) = chars.peek() {
        if c == '-' || c == '+' || c.is_ascii_digit() {
            num_str.push(c);
            chars.next();
        } else if c == '.' || c == 'e' || c == 'E' {
            is_float = true;
            num_str.push(c);
            chars.next();
        } else {
            break;
        }
    }

    if is_float {
        let f: f64 = num_str.parse().map_err(|e| format!("invalid float '{num_str}': {e}"))?;
        Ok(Value::Float(f))
    } else {
        let i: i64 = num_str.parse().map_err(|e| format!("invalid integer '{num_str}': {e}"))?;
        Ok(Value::Int(i))
    }
}

fn parse_bool(chars: &mut CharIter) -> Result<Value, String> {
    let mut word = String::new();
    while let Some(&(_, c)) = chars.peek() {
        if c.is_ascii_alphabetic() {
            word.push(c);
            chars.next();
        } else {
            break;
        }
    }
    match word.as_str() {
        "true" => Ok(Value::Bool(true)),
        "false" => Ok(Value::Bool(false)),
        _ => Err(format!("invalid boolean identifier '{word}'")),
    }
}

fn parse_null(chars: &mut CharIter) -> Result<Value, String> {
    let mut word = String::new();
    while let Some(&(_, c)) = chars.peek() {
        if c.is_ascii_alphabetic() {
            word.push(c);
            chars.next();
        } else {
            break;
        }
    }
    if word == "null" {
        Ok(Value::Unit)
    } else {
        Err(format!("invalid literal '{word}'"))
    }
}

fn parse_array(chars: &mut CharIter) -> Result<Value, String> {
    chars.next(); // consume '['
    skip_whitespace(chars);

    let mut list = Vec::new();
    if let Some(&(_, ']')) = chars.peek() {
        chars.next();
        return Ok(Value::List(list));
    }

    loop {
        skip_whitespace(chars);
        let val = parse_value(chars)?;
        list.push(val);
        skip_whitespace(chars);

        match chars.peek() {
            Some(&(_, ',')) => {
                chars.next();
                skip_whitespace(chars);
                // Allow trailing comma
                if let Some(&(_, ']')) = chars.peek() {
                    chars.next();
                    break;
                }
            }
            Some(&(_, ']')) => {
                chars.next();
                break;
            }
            Some(&(_, c)) => return Err(format!("expected ',' or ']' in array, found '{c}'")),
            None => return Err("unclosed array in JSON".into()),
        }
    }

    Ok(Value::List(list))
}

fn parse_object(chars: &mut CharIter) -> Result<Value, String> {
    chars.next(); // consume '{'
    skip_whitespace(chars);

    let mut map = HashMap::new();
    if let Some(&(_, '}')) = chars.peek() {
        chars.next();
        return Ok(Value::Map(map));
    }

    loop {
        skip_whitespace(chars);
        let (_, c) = chars.peek().cloned().ok_or_else(|| "unclosed object in JSON".to_string())?;
        if c != '"' {
            return Err(format!("expected string key in object, found '{c}'"));
        }
        let key = parse_string(chars)?;
        skip_whitespace(chars);

        let (_, colon) = chars.next().ok_or_else(|| "expected ':' after key in object".to_string())?;
        if colon != ':' {
            return Err(format!("expected ':', found '{colon}'"));
        }

        skip_whitespace(chars);
        let val = parse_value(chars)?;
        map.insert(key, val);

        skip_whitespace(chars);
        match chars.peek() {
            Some(&(_, ',')) => {
                chars.next();
                skip_whitespace(chars);
                // Allow trailing comma
                if let Some(&(_, '}')) = chars.peek() {
                    chars.next();
                    break;
                }
            }
            Some(&(_, '}')) => {
                chars.next();
                break;
            }
            Some(&(_, c)) => return Err(format!("expected ',' or '}}' in object, found '{c}'")),
            None => return Err("unclosed object in JSON".into()),
        }
    }

    Ok(Value::Map(map))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_stringify_primitives() {
        assert_eq!(parse("42").unwrap(), Value::Int(42));
        assert_eq!(parse("-100").unwrap(), Value::Int(-100));
        assert_eq!(parse("true").unwrap(), Value::Bool(true));
        assert_eq!(parse("false").unwrap(), Value::Bool(false));
        assert_eq!(parse("null").unwrap(), Value::Unit);
        assert_eq!(parse("\"hello world\"").unwrap(), Value::Str("hello world".into()));

        assert_eq!(stringify(&Value::Int(42)), "42");
        assert_eq!(stringify(&Value::Bool(true)), "true");
        assert_eq!(stringify(&Value::Unit), "null");
        assert_eq!(stringify(&Value::Str("foo".into())), "\"foo\"");
    }

    #[test]
    fn test_parse_and_stringify_containers() {
        let json_src = r#"{"name":"Can","roles":["lead","architect"],"active":true,"age":28}"#;
        let val = parse(json_src).unwrap();

        assert_eq!(get_path(&val, "name").unwrap(), Value::Str("Can".into()));
        assert_eq!(get_path(&val, "roles.0").unwrap(), Value::Str("lead".into()));
        assert_eq!(get_path(&val, "roles.1").unwrap(), Value::Str("architect".into()));
        assert_eq!(get_path(&val, "active").unwrap(), Value::Bool(true));
        assert_eq!(get_path(&val, "age").unwrap(), Value::Int(28));

        let reserialized = stringify(&val);
        let reparsed = parse(&reserialized).unwrap();
        assert_eq!(val, reparsed);
    }

    #[test]
    fn test_json_set_and_get() {
        let mut doc = parse(r#"{"user":{"name":"Can"}}"#).unwrap();
        set_path(&mut doc, "user.city", Value::Str("Istanbul".into())).unwrap();
        set_path(&mut doc, "user.score", Value::Int(100)).unwrap();

        assert_eq!(get_path(&doc, "user.city").unwrap(), Value::Str("Istanbul".into()));
        assert_eq!(get_path(&doc, "user.score").unwrap(), Value::Int(100));
    }
}
