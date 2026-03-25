use crate::type_system::Value;
use crate::{CorvoError, CorvoResult};

pub fn call_string_method(name: &str, args: &[Value]) -> CorvoResult<Value> {
    let method = name.strip_prefix("string.").unwrap();
    let target = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::r#type("string method requires a string target"))?;

    match method {
        "concat" => {
            let s2 = args
                .get(1)
                .and_then(|v| v.as_string())
                .map(|s| s.as_str())
                .unwrap_or("");
            Ok(Value::String(format!("{}{}", target, s2)))
        }
        "replace" => {
            let old = args
                .get(1)
                .and_then(|v| v.as_string())
                .map(|s| s.as_str())
                .unwrap_or("");
            let new = args
                .get(2)
                .and_then(|v| v.as_string())
                .map(|s| s.as_str())
                .unwrap_or("");
            Ok(Value::String(target.replace(old, new)))
        }
        "split" => {
            let delimiter = args
                .get(1)
                .and_then(|v| v.as_string())
                .map(|s| s.as_str())
                .unwrap_or("");
            let parts: Vec<Value> = target
                .split(delimiter)
                .map(|s| Value::String(s.to_string()))
                .collect();
            Ok(Value::List(parts))
        }
        "trim" => Ok(Value::String(target.trim().to_string())),
        "contains" => {
            let substr = args
                .get(1)
                .and_then(|v| v.as_string())
                .map(|s| s.as_str())
                .unwrap_or("");
            Ok(Value::Boolean(target.contains(substr)))
        }
        "starts_with" => {
            let prefix = args
                .get(1)
                .and_then(|v| v.as_string())
                .map(|s| s.as_str())
                .unwrap_or("");
            Ok(Value::Boolean(target.starts_with(prefix)))
        }
        "ends_with" => {
            let suffix = args
                .get(1)
                .and_then(|v| v.as_string())
                .map(|s| s.as_str())
                .unwrap_or("");
            Ok(Value::Boolean(target.ends_with(suffix)))
        }
        "to_lower" => Ok(Value::String(target.to_lowercase())),
        "to_upper" => Ok(Value::String(target.to_uppercase())),
        "len" => Ok(Value::Number(target.len() as f64)),
        "reverse" => Ok(Value::String(target.chars().rev().collect())),
        "is_empty" => Ok(Value::Boolean(target.is_empty())),
        _ => Err(CorvoError::unknown_function(format!("string.{}", method))),
    }
}

pub fn call_number_method(name: &str, args: &[Value]) -> CorvoResult<Value> {
    let method = name.strip_prefix("number.").unwrap();
    let target = args.first().and_then(|v| v.as_number()).unwrap_or(0.0);

    match method {
        "to_string" => Ok(Value::String(target.to_string())),
        "parse" => {
            let s = args
                .get(1)
                .and_then(|v| v.as_string())
                .map(|s| s.as_str())
                .unwrap_or("");
            s.parse::<f64>()
                .map(Value::Number)
                .map_err(|_| CorvoError::runtime("Failed to parse number".to_string()))
        }
        "is_nan" => Ok(Value::Boolean(target.is_nan())),
        "is_infinite" => Ok(Value::Boolean(target.is_infinite())),
        "is_finite" => Ok(Value::Boolean(target.is_finite())),
        "abs" => Ok(Value::Number(target.abs())),
        "floor" => Ok(Value::Number(target.floor())),
        "ceil" => Ok(Value::Number(target.ceil())),
        "round" => Ok(Value::Number(target.round())),
        "sqrt" => {
            if target < 0.0 {
                return Err(CorvoError::runtime(
                    "Cannot take square root of negative number".to_string(),
                ));
            }
            Ok(Value::Number(target.sqrt()))
        }
        _ => Err(CorvoError::unknown_function(format!("number.{}", method))),
    }
}

pub fn call_list_method(name: &str, args: &[Value]) -> CorvoResult<Value> {
    let method = name.strip_prefix("list.").unwrap();
    let target = args
        .first()
        .and_then(|v| v.as_list())
        .cloned()
        .unwrap_or_default();

    match method {
        "push" => {
            let item = args.get(1).cloned().unwrap_or(Value::Null);
            let mut new_list = target.clone();
            new_list.push(item);
            Ok(Value::List(new_list))
        }
        "pop" => {
            let mut new_list = target.clone();
            new_list.pop();
            Ok(Value::List(new_list))
        }
        "get" => {
            let index = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0) as usize;
            target
                .get(index)
                .cloned()
                .ok_or_else(|| CorvoError::runtime("Index out of bounds".to_string()))
        }
        "set" => {
            let index = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0) as usize;
            let value = args.get(2).cloned().unwrap_or(Value::Null);
            if index >= target.len() {
                return Err(CorvoError::runtime("Index out of bounds".to_string()));
            }
            let mut new_list = target.clone();
            new_list[index] = value;
            Ok(Value::List(new_list))
        }
        "first" => target
            .first()
            .cloned()
            .ok_or_else(|| CorvoError::runtime("List is empty".to_string())),
        "last" => target
            .last()
            .cloned()
            .ok_or_else(|| CorvoError::runtime("List is empty".to_string())),
        "len" => Ok(Value::Number(target.len() as f64)),
        "is_empty" => Ok(Value::Boolean(target.is_empty())),
        "contains" => {
            let item = args.get(1).cloned().unwrap_or(Value::Null);
            Ok(Value::Boolean(target.contains(&item)))
        }
        "reverse" => {
            let mut new_list = target.clone();
            new_list.reverse();
            Ok(Value::List(new_list))
        }
        "join" => {
            let delimiter = args
                .get(1)
                .and_then(|v| v.as_string())
                .map(|s| s.as_str())
                .unwrap_or("");
            let parts: Vec<String> = target.iter().map(|v| v.to_string()).collect();
            Ok(Value::String(parts.join(delimiter)))
        }
        "new" => Ok(Value::List(Vec::new())),
        "delete" => {
            let index = args.get(1).and_then(|v| v.as_number()).ok_or_else(|| {
                CorvoError::runtime("list.delete requires an index argument".to_string())
            })? as usize;
            if index >= target.len() {
                return Err(CorvoError::runtime("Index out of bounds".to_string()));
            }
            let mut new_list = target.clone();
            new_list.remove(index);
            Ok(Value::List(new_list))
        }
        "sort" => {
            let mut new_list = target.clone();
            new_list.sort_by_key(|a| a.to_string());
            Ok(Value::List(new_list))
        }
        "find" => {
            let item = args.get(1).cloned().unwrap_or(Value::Null);
            let index = target.iter().position(|v| v == &item);
            Ok(index
                .map(|i| Value::Number(i as f64))
                .unwrap_or(Value::Number(-1.0)))
        }
        "slice" => {
            let start = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0) as usize;
            let end = args
                .get(2)
                .and_then(|v| v.as_number())
                .map(|n| n as usize)
                .unwrap_or(target.len());
            let end = end.min(target.len());
            let start = start.min(end);
            Ok(Value::List(target[start..end].to_vec()))
        }
        "unique" => {
            let mut seen = std::collections::HashSet::new();
            let new_list: Vec<Value> = target
                .iter()
                .filter(|v| seen.insert(v.to_string()))
                .cloned()
                .collect();
            Ok(Value::List(new_list))
        }
        "flatten" => {
            let mut new_list = Vec::new();
            for item in &target {
                if let Value::List(inner) = item {
                    new_list.extend(inner.iter().cloned());
                } else {
                    new_list.push(item.clone());
                }
            }
            Ok(Value::List(new_list))
        }
        _ => Err(CorvoError::unknown_function(format!("list.{}", method))),
    }
}

pub fn call_map_method(name: &str, args: &[Value]) -> CorvoResult<Value> {
    let method = name.strip_prefix("map.").unwrap();
    let target = args
        .first()
        .and_then(|v| v.as_map())
        .cloned()
        .unwrap_or_default();

    match method {
        "keys" => {
            let keys: Vec<Value> = target.keys().map(|k| Value::String(k.clone())).collect();
            Ok(Value::List(keys))
        }
        "values" => Ok(Value::List(target.values().cloned().collect())),
        "len" => Ok(Value::Number(target.len() as f64)),
        "is_empty" => Ok(Value::Boolean(target.is_empty())),
        "has_key" => {
            let key = args
                .get(1)
                .and_then(|v| v.as_string())
                .map(|s| s.as_str())
                .unwrap_or("");
            Ok(Value::Boolean(target.contains_key(key)))
        }
        "get" => {
            let key = args
                .get(1)
                .and_then(|v| v.as_string())
                .map(|s| s.as_str())
                .unwrap_or("");
            let default = args.get(2).cloned().unwrap_or(Value::Null);
            Ok(target.get(key).cloned().unwrap_or(default))
        }
        "set" => {
            let key = args
                .get(1)
                .and_then(|v| v.as_string())
                .map(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            let value = args.get(2).cloned().unwrap_or(Value::Null);
            let mut new_map = target.clone();
            new_map.insert(key, value);
            Ok(Value::Map(new_map))
        }
        "remove" | "delete" => {
            let key = args
                .get(1)
                .and_then(|v| v.as_string())
                .map(|s| s.as_str())
                .unwrap_or("");
            let mut new_map = target.clone();
            new_map.remove(key);
            Ok(Value::Map(new_map))
        }
        "merge" => {
            let other = args
                .get(1)
                .and_then(|v| v.as_map())
                .cloned()
                .unwrap_or_default();
            let mut new_map = target.clone();
            new_map.extend(other);
            Ok(Value::Map(new_map))
        }
        "has" => {
            let key = args
                .get(1)
                .and_then(|v| v.as_string())
                .map(|s| s.as_str())
                .unwrap_or("");
            Ok(Value::Boolean(target.contains_key(key)))
        }
        "entries" => {
            let mut entries: Vec<Value> = target
                .iter()
                .map(|(k, v)| {
                    let mut entry = std::collections::HashMap::new();
                    entry.insert("key".to_string(), Value::String(k.clone()));
                    entry.insert("value".to_string(), v.clone());
                    Value::Map(entry)
                })
                .collect();
            entries.sort_by(|a, b| {
                let ka = if let Value::Map(m) = a {
                    m.get("key").map(|v| v.to_string()).unwrap_or_default()
                } else {
                    String::new()
                };
                let kb = if let Value::Map(m) = b {
                    m.get("key").map(|v| v.to_string()).unwrap_or_default()
                } else {
                    String::new()
                };
                ka.cmp(&kb)
            });
            Ok(Value::List(entries))
        }
        "new" => Ok(Value::Map(std::collections::HashMap::new())),
        _ => Err(CorvoError::unknown_function(format!("map.{}", method))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_concat() {
        let args = vec![
            Value::String("hello".to_string()),
            Value::String(" world".to_string()),
        ];
        let result = call_string_method("string.concat", &args).unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));
    }

    #[test]
    fn test_string_replace() {
        let args = vec![
            Value::String("hello world".to_string()),
            Value::String("world".to_string()),
            Value::String("rust".to_string()),
        ];
        let result = call_string_method("string.replace", &args).unwrap();
        assert_eq!(result, Value::String("hello rust".to_string()));
    }

    #[test]
    fn test_string_split() {
        let args = vec![
            Value::String("a,b,c".to_string()),
            Value::String(",".to_string()),
        ];
        let result = call_string_method("string.split", &args).unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::String("a".to_string()));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_string_trim() {
        let args = vec![Value::String("  hello  ".to_string())];
        let result = call_string_method("string.trim", &args).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_string_contains() {
        let args = vec![
            Value::String("hello world".to_string()),
            Value::String("world".to_string()),
        ];
        let result = call_string_method("string.contains", &args).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_string_to_lower() {
        let args = vec![Value::String("HELLO".to_string())];
        let result = call_string_method("string.to_lower", &args).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_string_to_upper() {
        let args = vec![Value::String("hello".to_string())];
        let result = call_string_method("string.to_upper", &args).unwrap();
        assert_eq!(result, Value::String("HELLO".to_string()));
    }

    #[test]
    fn test_string_len() {
        let args = vec![Value::String("hello".to_string())];
        let result = call_string_method("string.len", &args).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_number_to_string() {
        let args = vec![Value::Number(42.5)];
        let result = call_number_method("number.to_string", &args).unwrap();
        assert_eq!(result, Value::String("42.5".to_string()));
    }

    #[test]
    fn test_number_parse() {
        let args = vec![Value::Number(0.0), Value::String("1.5".to_string())];
        let result = call_number_method("number.parse", &args).unwrap();
        assert_eq!(result, Value::Number(1.5));
    }

    #[test]
    fn test_number_is_nan() {
        let args = vec![Value::Number(f64::NAN)];
        let result = call_number_method("number.is_nan", &args).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_list_push() {
        let args = vec![
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::Number(3.0),
        ];
        let result = call_list_method("list.push", &args).unwrap();
        match result {
            Value::List(items) => assert_eq!(items.len(), 3),
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_list_pop() {
        let args = vec![Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ])];
        let result = call_list_method("list.pop", &args).unwrap();
        match result {
            Value::List(items) => assert_eq!(items.len(), 2),
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_list_get() {
        let args = vec![
            Value::List(vec![
                Value::String("a".to_string()),
                Value::String("b".to_string()),
            ]),
            Value::Number(1.0),
        ];
        let result = call_list_method("list.get", &args).unwrap();
        assert_eq!(result, Value::String("b".to_string()));
    }

    #[test]
    fn test_list_len() {
        let args = vec![Value::List(vec![Value::Number(1.0), Value::Number(2.0)])];
        let result = call_list_method("list.len", &args).unwrap();
        assert_eq!(result, Value::Number(2.0));
    }

    #[test]
    fn test_list_contains() {
        let args = vec![
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::Number(2.0),
        ];
        let result = call_list_method("list.contains", &args).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_list_join() {
        let args = vec![
            Value::List(vec![
                Value::String("a".to_string()),
                Value::String("b".to_string()),
                Value::String("c".to_string()),
            ]),
            Value::String(", ".to_string()),
        ];
        let result = call_list_method("list.join", &args).unwrap();
        assert_eq!(result, Value::String("a, b, c".to_string()));
    }

    #[test]
    fn test_map_keys() {
        let mut map = std::collections::HashMap::new();
        map.insert("a".to_string(), Value::Number(1.0));
        map.insert("b".to_string(), Value::Number(2.0));
        let args = vec![Value::Map(map)];
        let result = call_map_method("map.keys", &args).unwrap();
        match result {
            Value::List(keys) => assert_eq!(keys.len(), 2),
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_map_has_key() {
        let mut map = std::collections::HashMap::new();
        map.insert("key".to_string(), Value::Number(1.0));
        let args = vec![Value::Map(map), Value::String("key".to_string())];
        let result = call_map_method("map.has_key", &args).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_map_get() {
        let mut map = std::collections::HashMap::new();
        map.insert("key".to_string(), Value::Number(42.0));
        let args = vec![Value::Map(map), Value::String("key".to_string())];
        let result = call_map_method("map.get", &args).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_map_set() {
        let mut map = std::collections::HashMap::new();
        map.insert("a".to_string(), Value::Number(1.0));
        let args = vec![
            Value::Map(map),
            Value::String("b".to_string()),
            Value::Number(2.0),
        ];
        let result = call_map_method("map.set", &args).unwrap();
        match result {
            Value::Map(m) => assert_eq!(m.len(), 2),
            _ => panic!("Expected Map"),
        }
    }

    #[test]
    fn test_unknown_string_method() {
        let args = vec![Value::String("test".to_string())];
        let result = call_string_method("string.unknown", &args);
        assert!(result.is_err());
    }

    // --- New String Method Tests ---

    #[test]
    fn test_string_starts_with() {
        let args = vec![
            Value::String("hello world".to_string()),
            Value::String("hello".to_string()),
        ];
        assert_eq!(
            call_string_method("string.starts_with", &args).unwrap(),
            Value::Boolean(true)
        );

        let args = vec![
            Value::String("hello world".to_string()),
            Value::String("world".to_string()),
        ];
        assert_eq!(
            call_string_method("string.starts_with", &args).unwrap(),
            Value::Boolean(false)
        );
    }

    #[test]
    fn test_string_ends_with() {
        let args = vec![
            Value::String("hello world".to_string()),
            Value::String("world".to_string()),
        ];
        assert_eq!(
            call_string_method("string.ends_with", &args).unwrap(),
            Value::Boolean(true)
        );
    }

    #[test]
    fn test_string_reverse() {
        let args = vec![Value::String("hello".to_string())];
        assert_eq!(
            call_string_method("string.reverse", &args).unwrap(),
            Value::String("olleh".to_string())
        );
    }

    #[test]
    fn test_string_is_empty() {
        assert_eq!(
            call_string_method("string.is_empty", &[Value::String("".to_string())]).unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            call_string_method("string.is_empty", &[Value::String("a".to_string())]).unwrap(),
            Value::Boolean(false)
        );
    }

    // --- New Number Method Tests ---

    #[test]
    fn test_number_abs() {
        assert_eq!(
            call_number_method("number.abs", &[Value::Number(-5.0)]).unwrap(),
            Value::Number(5.0)
        );
        assert_eq!(
            call_number_method("number.abs", &[Value::Number(5.0)]).unwrap(),
            Value::Number(5.0)
        );
    }

    #[test]
    fn test_number_floor() {
        assert_eq!(
            call_number_method("number.floor", &[Value::Number(3.7)]).unwrap(),
            Value::Number(3.0)
        );
        assert_eq!(
            call_number_method("number.floor", &[Value::Number(-3.2)]).unwrap(),
            Value::Number(-4.0)
        );
    }

    #[test]
    fn test_number_ceil() {
        assert_eq!(
            call_number_method("number.ceil", &[Value::Number(3.2)]).unwrap(),
            Value::Number(4.0)
        );
        assert_eq!(
            call_number_method("number.ceil", &[Value::Number(-3.7)]).unwrap(),
            Value::Number(-3.0)
        );
    }

    #[test]
    fn test_number_round() {
        assert_eq!(
            call_number_method("number.round", &[Value::Number(3.5)]).unwrap(),
            Value::Number(4.0)
        );
        assert_eq!(
            call_number_method("number.round", &[Value::Number(3.4)]).unwrap(),
            Value::Number(3.0)
        );
    }

    #[test]
    fn test_number_sqrt() {
        assert_eq!(
            call_number_method("number.sqrt", &[Value::Number(9.0)]).unwrap(),
            Value::Number(3.0)
        );
        assert!(call_number_method("number.sqrt", &[Value::Number(-1.0)]).is_err());
    }

    #[test]
    fn test_number_is_infinite() {
        assert_eq!(
            call_number_method("number.is_infinite", &[Value::Number(f64::INFINITY)]).unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            call_number_method("number.is_infinite", &[Value::Number(42.0)]).unwrap(),
            Value::Boolean(false)
        );
    }

    #[test]
    fn test_number_is_finite() {
        assert_eq!(
            call_number_method("number.is_finite", &[Value::Number(42.0)]).unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            call_number_method("number.is_finite", &[Value::Number(f64::INFINITY)]).unwrap(),
            Value::Boolean(false)
        );
    }

    // --- New List Method Tests ---

    #[test]
    fn test_list_set() {
        let args = vec![
            Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
            ]),
            Value::Number(1.0),
            Value::Number(99.0),
        ];
        let result = call_list_method("list.set", &args).unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[1], Value::Number(99.0));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_list_set_out_of_bounds() {
        let args = vec![
            Value::List(vec![Value::Number(1.0)]),
            Value::Number(5.0),
            Value::Number(99.0),
        ];
        assert!(call_list_method("list.set", &args).is_err());
    }

    #[test]
    fn test_list_first() {
        let args = vec![Value::List(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        ])];
        assert_eq!(
            call_list_method("list.first", &args).unwrap(),
            Value::String("a".to_string())
        );
    }

    #[test]
    fn test_list_first_empty() {
        let args = vec![Value::List(vec![])];
        assert!(call_list_method("list.first", &args).is_err());
    }

    #[test]
    fn test_list_last() {
        let args = vec![Value::List(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        ])];
        assert_eq!(
            call_list_method("list.last", &args).unwrap(),
            Value::String("b".to_string())
        );
    }

    #[test]
    fn test_list_reverse() {
        let args = vec![Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ])];
        let result = call_list_method("list.reverse", &args).unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items[0], Value::Number(3.0));
                assert_eq!(items[2], Value::Number(1.0));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_list_is_empty() {
        assert_eq!(
            call_list_method("list.is_empty", &[Value::List(vec![])]).unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            call_list_method("list.is_empty", &[Value::List(vec![Value::Number(1.0)])]).unwrap(),
            Value::Boolean(false)
        );
    }

    #[test]
    fn test_list_contains_not_found() {
        let args = vec![
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::Number(99.0),
        ];
        assert_eq!(
            call_list_method("list.contains", &args).unwrap(),
            Value::Boolean(false)
        );
    }

    #[test]
    fn test_list_get_out_of_bounds() {
        let args = vec![Value::List(vec![Value::Number(1.0)]), Value::Number(5.0)];
        assert!(call_list_method("list.get", &args).is_err());
    }

    #[test]
    fn test_unknown_list_method() {
        let args = vec![Value::List(vec![])];
        assert!(call_list_method("list.unknown", &args).is_err());
    }

    // --- New Map Method Tests ---

    #[test]
    fn test_map_remove() {
        let mut map = std::collections::HashMap::new();
        map.insert("a".to_string(), Value::Number(1.0));
        map.insert("b".to_string(), Value::Number(2.0));
        let args = vec![Value::Map(map), Value::String("a".to_string())];
        let result = call_map_method("map.remove", &args).unwrap();
        match result {
            Value::Map(m) => {
                assert_eq!(m.len(), 1);
                assert!(!m.contains_key("a"));
                assert!(m.contains_key("b"));
            }
            _ => panic!("Expected Map"),
        }
    }

    #[test]
    fn test_map_merge() {
        let mut m1 = std::collections::HashMap::new();
        m1.insert("a".to_string(), Value::Number(1.0));
        let mut m2 = std::collections::HashMap::new();
        m2.insert("b".to_string(), Value::Number(2.0));
        let args = vec![Value::Map(m1), Value::Map(m2)];
        let result = call_map_method("map.merge", &args).unwrap();
        match result {
            Value::Map(m) => {
                assert_eq!(m.len(), 2);
            }
            _ => panic!("Expected Map"),
        }
    }

    #[test]
    fn test_map_len() {
        let mut map = std::collections::HashMap::new();
        map.insert("a".to_string(), Value::Number(1.0));
        map.insert("b".to_string(), Value::Number(2.0));
        assert_eq!(
            call_map_method("map.len", &[Value::Map(map)]).unwrap(),
            Value::Number(2.0)
        );
    }

    #[test]
    fn test_map_is_empty() {
        assert_eq!(
            call_map_method(
                "map.is_empty",
                &[Value::Map(std::collections::HashMap::new())]
            )
            .unwrap(),
            Value::Boolean(true)
        );
        let mut map = std::collections::HashMap::new();
        map.insert("a".to_string(), Value::Number(1.0));
        assert_eq!(
            call_map_method("map.is_empty", &[Value::Map(map)]).unwrap(),
            Value::Boolean(false)
        );
    }

    #[test]
    fn test_map_values() {
        let mut map = std::collections::HashMap::new();
        map.insert("a".to_string(), Value::Number(1.0));
        let result = call_map_method("map.values", &[Value::Map(map)]).unwrap();
        match result {
            Value::List(values) => assert_eq!(values.len(), 1),
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_map_has_key_not_found() {
        let map = std::collections::HashMap::new();
        let args = vec![Value::Map(map), Value::String("missing".to_string())];
        assert_eq!(
            call_map_method("map.has_key", &args).unwrap(),
            Value::Boolean(false)
        );
    }

    #[test]
    fn test_map_get_default() {
        let map = std::collections::HashMap::new();
        let args = vec![
            Value::Map(map),
            Value::String("missing".to_string()),
            Value::String("default".to_string()),
        ];
        assert_eq!(
            call_map_method("map.get", &args).unwrap(),
            Value::String("default".to_string())
        );
    }

    #[test]
    fn test_unknown_map_method() {
        let args = vec![Value::Map(std::collections::HashMap::new())];
        assert!(call_map_method("map.unknown", &args).is_err());
    }
}
