use crate::type_system::Value;
use crate::{CorvoError, CorvoResult};
use std::collections::HashMap;

pub fn get_env(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let key = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("get_env requires a string key"))?;

    let default = args
        .get(1)
        .and_then(|v| v.as_string())
        .cloned()
        .unwrap_or_default();

    Ok(Value::String(std::env::var(key).unwrap_or(default)))
}

pub fn set_env(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let key = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("set_env requires a string key"))?;

    let value = args
        .get(1)
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("set_env requires a string value"))?;

    std::env::set_var(key, value);
    Ok(Value::Null)
}

pub fn exec(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let cmd = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("exec requires a command string"))?;

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| CorvoError::io(e.to_string()))?;

    let mut result = HashMap::new();
    result.insert(
        "stdout".to_string(),
        Value::String(String::from_utf8_lossy(&output.stdout).to_string()),
    );
    result.insert(
        "stderr".to_string(),
        Value::String(String::from_utf8_lossy(&output.stderr).to_string()),
    );
    result.insert(
        "code".to_string(),
        Value::Number(output.status.code().unwrap_or(-1) as f64),
    );

    Ok(Value::Map(result))
}

pub fn info(_args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let mut result = HashMap::new();
    result.insert(
        "os".to_string(),
        Value::String(std::env::consts::OS.to_string()),
    );
    result.insert(
        "arch".to_string(),
        Value::String(std::env::consts::ARCH.to_string()),
    );
    result.insert(
        "hostname".to_string(),
        Value::String(
            hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
        ),
    );

    Ok(Value::Map(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_args() -> HashMap<String, Value> {
        HashMap::new()
    }

    #[test]
    fn test_get_env_existing() {
        std::env::set_var("CORVO_TEST_VAR", "test_value");
        let args = vec![Value::String("CORVO_TEST_VAR".to_string())];
        assert_eq!(
            get_env(&args, &empty_args()).unwrap(),
            Value::String("test_value".to_string())
        );
        std::env::remove_var("CORVO_TEST_VAR");
    }

    #[test]
    fn test_get_env_default() {
        let args = vec![
            Value::String("CORVO_NONEXISTENT_VAR_XYZ".to_string()),
            Value::String("default_val".to_string()),
        ];
        assert_eq!(
            get_env(&args, &empty_args()).unwrap(),
            Value::String("default_val".to_string())
        );
    }

    #[test]
    fn test_get_env_empty_default() {
        let args = vec![Value::String("CORVO_NONEXISTENT_VAR_XYZ".to_string())];
        assert_eq!(
            get_env(&args, &empty_args()).unwrap(),
            Value::String("".to_string())
        );
    }

    #[test]
    fn test_set_env() {
        let args = vec![
            Value::String("CORVO_TEST_SET_XYZ".to_string()),
            Value::String("set_value".to_string()),
        ];
        assert_eq!(set_env(&args, &empty_args()).unwrap(), Value::Null);
        assert_eq!(std::env::var("CORVO_TEST_SET_XYZ").unwrap(), "set_value");
        std::env::remove_var("CORVO_TEST_SET_XYZ");
    }

    #[test]
    fn test_exec_echo() {
        let args = vec![Value::String("echo hello".to_string())];
        let result = exec(&args, &empty_args()).unwrap();
        match result {
            Value::Map(m) => {
                let stdout = m.get("stdout").unwrap().as_string().unwrap();
                assert!(stdout.contains("hello"));
                let code = m.get("code").unwrap().as_number().unwrap();
                assert_eq!(code, 0.0);
            }
            _ => panic!("Expected Map"),
        }
    }

    #[test]
    fn test_exec_returns_status_code() {
        let args = vec![Value::String("exit 42".to_string())];
        let result = exec(&args, &empty_args()).unwrap();
        match result {
            Value::Map(m) => {
                let code = m.get("code").unwrap().as_number().unwrap();
                assert_eq!(code, 42.0);
            }
            _ => panic!("Expected Map"),
        }
    }

    #[test]
    fn test_info() {
        let result = info(&[], &empty_args()).unwrap();
        match result {
            Value::Map(m) => {
                assert!(m.contains_key("os"));
                assert!(m.contains_key("arch"));
                assert!(m.contains_key("hostname"));
            }
            _ => panic!("Expected Map"),
        }
    }

    #[test]
    fn test_info_os_value() {
        let result = info(&[], &empty_args()).unwrap();
        match result {
            Value::Map(m) => {
                let os_val = m.get("os").unwrap().as_string().unwrap();
                assert!(!os_val.is_empty());
            }
            _ => panic!("Expected Map"),
        }
    }
}
