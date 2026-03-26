use crate::type_system::Value;
use crate::{CorvoError, CorvoResult};
use std::collections::HashMap;

/// Build a `regex::Regex` from a Corvo regex value's pattern and flags.
///
/// The following flags are supported:
/// * `i` – case-insensitive matching (`(?i)`)
/// * `m` – multi-line mode (`(?m)`)
/// * `s` – dot-matches-newline (`(?s)`)
/// * `g` – global (no-op here; use `re.find_all` instead of `re.find`)
/// * `u` – Unicode (already the default in Rust's regex crate; ignored)
pub fn build_regex(pattern: &str, flags: &str) -> CorvoResult<regex::Regex> {
    let mut prefix = String::new();
    for ch in flags.chars() {
        match ch {
            'i' => prefix.push_str("(?i)"),
            'm' => prefix.push_str("(?m)"),
            's' => prefix.push_str("(?s)"),
            'g' | 'u' => {} // g = global (use find_all), u = unicode (default)
            _ => {}         // ignore unknown flags
        }
    }
    let full_pattern = format!("{}{}", prefix, pattern);
    regex::Regex::new(&full_pattern)
        .map_err(|e| CorvoError::runtime(format!("Invalid regex pattern: {}", e)))
}

/// Extract (pattern, flags) from the first argument, which must be a regex value.
fn extract_regex(v: Option<&Value>) -> CorvoResult<(&str, &str)> {
    v.and_then(|val| val.as_regex())
        .map(|(p, f)| (p.as_str(), f.as_str()))
        .ok_or_else(|| CorvoError::r#type("re method requires a regex as the first argument"))
}

/// `re.match(regex, string)` – returns `true` if the string contains a match.
pub fn is_match(args: &[Value], _named: &HashMap<String, Value>) -> CorvoResult<Value> {
    let (pattern, flags) = extract_regex(args.first())?;
    let text = args
        .get(1)
        .and_then(|v| v.as_string())
        .map(|s| s.as_str())
        .ok_or_else(|| CorvoError::r#type("re.match requires a string as the second argument"))?;
    let re = build_regex(pattern, flags)?;
    Ok(Value::Boolean(re.is_match(text)))
}

/// `re.find(regex, string)` – returns the first match as a string, or null.
pub fn find(args: &[Value], _named: &HashMap<String, Value>) -> CorvoResult<Value> {
    let (pattern, flags) = extract_regex(args.first())?;
    let text = args
        .get(1)
        .and_then(|v| v.as_string())
        .map(|s| s.as_str())
        .ok_or_else(|| CorvoError::r#type("re.find requires a string as the second argument"))?;
    let re = build_regex(pattern, flags)?;
    Ok(re
        .find(text)
        .map(|m| Value::String(m.as_str().to_string()))
        .unwrap_or(Value::Null))
}

/// `re.find_all(regex, string)` – returns all non-overlapping matches as a list.
pub fn find_all(args: &[Value], _named: &HashMap<String, Value>) -> CorvoResult<Value> {
    let (pattern, flags) = extract_regex(args.first())?;
    let text = args
        .get(1)
        .and_then(|v| v.as_string())
        .map(|s| s.as_str())
        .ok_or_else(|| {
            CorvoError::r#type("re.find_all requires a string as the second argument")
        })?;
    let re = build_regex(pattern, flags)?;
    let matches: Vec<Value> = re
        .find_iter(text)
        .map(|m| Value::String(m.as_str().to_string()))
        .collect();
    Ok(Value::List(matches))
}

/// `re.replace(regex, string, replacement)` – replaces the first match.
pub fn replace(args: &[Value], _named: &HashMap<String, Value>) -> CorvoResult<Value> {
    let (pattern, flags) = extract_regex(args.first())?;
    let text = args
        .get(1)
        .and_then(|v| v.as_string())
        .map(|s| s.as_str())
        .ok_or_else(|| CorvoError::r#type("re.replace requires a string as the second argument"))?;
    let replacement = args
        .get(2)
        .and_then(|v| v.as_string())
        .map(|s| s.as_str())
        .unwrap_or("");
    let re = build_regex(pattern, flags)?;
    Ok(Value::String(re.replace(text, replacement).into_owned()))
}

/// `re.replace_all(regex, string, replacement)` – replaces all matches.
pub fn replace_all(args: &[Value], _named: &HashMap<String, Value>) -> CorvoResult<Value> {
    let (pattern, flags) = extract_regex(args.first())?;
    let text = args
        .get(1)
        .and_then(|v| v.as_string())
        .map(|s| s.as_str())
        .ok_or_else(|| {
            CorvoError::r#type("re.replace_all requires a string as the second argument")
        })?;
    let replacement = args
        .get(2)
        .and_then(|v| v.as_string())
        .map(|s| s.as_str())
        .unwrap_or("");
    let re = build_regex(pattern, flags)?;
    Ok(Value::String(
        re.replace_all(text, replacement).into_owned(),
    ))
}

/// `re.split(regex, string)` – splits a string by the regex and returns a list.
pub fn split(args: &[Value], _named: &HashMap<String, Value>) -> CorvoResult<Value> {
    let (pattern, flags) = extract_regex(args.first())?;
    let text = args
        .get(1)
        .and_then(|v| v.as_string())
        .map(|s| s.as_str())
        .ok_or_else(|| CorvoError::r#type("re.split requires a string as the second argument"))?;
    let re = build_regex(pattern, flags)?;
    let parts: Vec<Value> = re
        .split(text)
        .map(|s| Value::String(s.to_string()))
        .collect();
    Ok(Value::List(parts))
}

/// `re.new(pattern)` or `re.new(pattern, flags)` – creates a new regex value.
pub fn new_regex(args: &[Value], _named: &HashMap<String, Value>) -> CorvoResult<Value> {
    let pattern = args
        .first()
        .and_then(|v| v.as_string())
        .map(|s| s.as_str())
        .ok_or_else(|| CorvoError::r#type("re.new requires a pattern string"))?;
    let flags = args
        .get(1)
        .and_then(|v| v.as_string())
        .map(|s| s.as_str())
        .unwrap_or("");
    // Validate the pattern up-front.
    build_regex(pattern, flags)?;
    Ok(Value::Regex(pattern.to_string(), flags.to_string()))
}
