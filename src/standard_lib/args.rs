//! Minimal command-line token scan for Corvo scripts (`args.scan`).
//!
//! Rules:
//! - `--` ends option parsing; remaining tokens are positional.
//! - `--name=value` sets option `name` to `value` (empty value allowed).
//! - `--name` sets `name` to the next token if it does not start with `-`, else `true`.
//! - `-abc` (one hyphen, multiple letters) sets each letter as a boolean flag.
//! - A lone `-` is treated as a positional token.
//! - Duplicate option keys: last wins.

use crate::type_system::Value;
use crate::{CorvoError, CorvoResult};
use std::collections::HashMap;

fn strings_from_argv_list(arg: &Value) -> CorvoResult<Vec<String>> {
    let list = arg.as_list().ok_or_else(|| {
        CorvoError::invalid_argument("args.scan requires a list of strings (e.g. os.argv())")
    })?;
    let mut out = Vec::with_capacity(list.len());
    for v in list {
        let s = v.as_string().ok_or_else(|| {
            CorvoError::invalid_argument("args.scan: argv list must contain only strings")
        })?;
        out.push(s.clone());
    }
    Ok(out)
}

/// Parse a list of argv strings into `positional` (list) and `options` (map: string → bool or string).
pub fn scan(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let tokens = if args.is_empty() {
        Vec::new()
    } else {
        strings_from_argv_list(&args[0])?
    };

    let mut positional: Vec<Value> = Vec::new();
    let mut options: HashMap<String, Value> = HashMap::new();

    let mut i = 0usize;
    while i < tokens.len() {
        let tok = &tokens[i];

        if tok == "--" {
            for t in tokens.iter().skip(i + 1) {
                positional.push(Value::String(t.clone()));
            }
            break;
        }

        if let Some(body) = tok.strip_prefix("--") {
            if let Some(eq) = body.find('=') {
                let name = body[..eq].to_string();
                let val = body[eq + 1..].to_string();
                if !name.is_empty() {
                    options.insert(name, Value::String(val));
                } else {
                    positional.push(Value::String(tok.clone()));
                }
                i += 1;
                continue;
            }

            if body.is_empty() {
                positional.push(Value::String(tok.clone()));
                i += 1;
                continue;
            }

            let name = body.to_string();
            if i + 1 < tokens.len() && !tokens[i + 1].starts_with('-') {
                options.insert(name, Value::String(tokens[i + 1].clone()));
                i += 2;
            } else {
                options.insert(name, Value::Boolean(true));
                i += 1;
            }
            continue;
        }

        if tok.starts_with('-') && tok.len() > 1 {
            for ch in tok[1..].chars() {
                options.insert(ch.to_string(), Value::Boolean(true));
            }
            i += 1;
            continue;
        }

        positional.push(Value::String(tok.clone()));
        i += 1;
    }

    let mut result = HashMap::new();
    result.insert("positional".to_string(), Value::List(positional));
    result.insert("options".to_string(), Value::Map(options));
    Ok(Value::Map(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_named() -> HashMap<String, Value> {
        HashMap::new()
    }

    fn argv(vals: &[&str]) -> Value {
        Value::List(
            vals.iter()
                .map(|s| Value::String((*s).to_string()))
                .collect(),
        )
    }

    fn run_scan(argv_val: Value) -> (Vec<String>, HashMap<String, Value>) {
        let m = scan(&[argv_val], &empty_named()).unwrap();
        let map = m.as_map().unwrap();
        let pos = map
            .get("positional")
            .unwrap()
            .as_list()
            .unwrap()
            .iter()
            .map(|v| v.as_string().unwrap().clone())
            .collect();
        let opts = map.get("options").unwrap().as_map().unwrap().clone();
        (pos, opts)
    }

    #[test]
    fn empty_argv() {
        let (pos, opts) = run_scan(argv(&[]));
        assert!(pos.is_empty());
        assert!(opts.is_empty());
    }

    #[test]
    fn positionals_only() {
        let (pos, opts) = run_scan(argv(&["a", "b"]));
        assert_eq!(pos, vec!["a", "b"]);
        assert!(opts.is_empty());
    }

    #[test]
    fn double_dash_rest_positional() {
        let (pos, opts) = run_scan(argv(&["--foo", "--", "-x", "y"]));
        assert_eq!(pos, vec!["-x", "y"]);
        assert_eq!(opts.get("foo"), Some(&Value::Boolean(true)));
    }

    #[test]
    fn long_equals() {
        let (_pos, opts) = run_scan(argv(&["--out=file.txt"]));
        assert_eq!(
            opts.get("out"),
            Some(&Value::String("file.txt".to_string()))
        );
    }

    #[test]
    fn long_empty_equals() {
        let (_pos, opts) = run_scan(argv(&["--tag="]));
        assert_eq!(opts.get("tag"), Some(&Value::String("".to_string())));
    }

    #[test]
    fn long_takes_next() {
        let (pos, opts) = run_scan(argv(&["--out", "path"]));
        assert!(pos.is_empty());
        assert_eq!(opts.get("out"), Some(&Value::String("path".to_string())));
    }

    #[test]
    fn long_bool_when_next_is_flag() {
        let (_pos, opts) = run_scan(argv(&["--verbose", "--other"]));
        assert_eq!(opts.get("verbose"), Some(&Value::Boolean(true)));
        assert_eq!(opts.get("other"), Some(&Value::Boolean(true)));
    }

    #[test]
    fn short_cluster() {
        let (_pos, opts) = run_scan(argv(&["-abc"]));
        assert_eq!(opts.get("a"), Some(&Value::Boolean(true)));
        assert_eq!(opts.get("b"), Some(&Value::Boolean(true)));
        assert_eq!(opts.get("c"), Some(&Value::Boolean(true)));
    }

    #[test]
    fn lone_hyphen_positional() {
        let (pos, opts) = run_scan(argv(&["-"]));
        assert_eq!(pos, vec!["-"]);
        assert!(opts.is_empty());
    }

    #[test]
    fn duplicate_last_wins() {
        let (_pos, opts) = run_scan(argv(&["--x=1", "--x=2"]));
        assert_eq!(opts.get("x"), Some(&Value::String("2".to_string())));
    }

    #[test]
    fn missing_arg_errors() {
        assert!(scan(&[], &empty_named()).is_ok());
        let err = scan(&[Value::Number(1.0)], &empty_named()).unwrap_err();
        assert!(format!("{}", err).contains("list"));
    }
}
