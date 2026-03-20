use crate::type_system::Value;
use crate::{CorvoError, CorvoResult};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn read(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let path = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("fs.read requires a path"))?;

    fs::read_to_string(path)
        .map(Value::String)
        .map_err(|e| CorvoError::file_system(e.to_string()))
}

pub fn write(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let path = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("fs.write requires a path"))?;

    let content = args
        .get(1)
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("fs.write requires content"))?;

    fs::write(path, content)
        .map(|_| Value::Boolean(true))
        .map_err(|e| CorvoError::file_system(e.to_string()))
}

pub fn append(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let path = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("fs.append requires a path"))?;

    let content = args
        .get(1)
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("fs.append requires content"))?;

    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, content.as_bytes()))
        .map(|_| Value::Boolean(true))
        .map_err(|e| CorvoError::file_system(e.to_string()))
}

pub fn delete(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let path = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("fs.delete requires a path"))?;

    if Path::new(path).is_dir() {
        fs::remove_dir_all(path)
            .map(|_| Value::Boolean(true))
            .map_err(|e| CorvoError::file_system(e.to_string()))
    } else {
        fs::remove_file(path)
            .map(|_| Value::Boolean(true))
            .map_err(|e| CorvoError::file_system(e.to_string()))
    }
}

pub fn exists(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let path = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("fs.exists requires a path"))?;

    Ok(Value::Boolean(Path::new(path).exists()))
}

pub fn mkdir(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let path = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("fs.mkdir requires a path"))?;

    let recursive = args.get(1).and_then(|v| v.as_bool()).unwrap_or(false);

    if recursive {
        fs::create_dir_all(path)
            .map(|_| Value::Boolean(true))
            .map_err(|e| CorvoError::file_system(e.to_string()))
    } else {
        fs::create_dir(path)
            .map(|_| Value::Boolean(true))
            .map_err(|e| CorvoError::file_system(e.to_string()))
    }
}

pub fn list_dir(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let path = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("fs.list_dir requires a path"))?;

    let entries = fs::read_dir(path)
        .map_err(|e| CorvoError::file_system(e.to_string()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| Value::String(entry.file_name().to_string_lossy().to_string()))
        .collect();

    Ok(Value::List(entries))
}

pub fn copy(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let src = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("fs.copy requires a source path"))?;

    let dest = args
        .get(1)
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("fs.copy requires a destination path"))?;

    fs::copy(src, dest)
        .map(|_| Value::Boolean(true))
        .map_err(|e| CorvoError::file_system(e.to_string()))
}

pub fn move_file(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let src = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("fs.move requires a source path"))?;

    let dest = args
        .get(1)
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("fs.move requires a destination path"))?;

    fs::rename(src, dest)
        .map(|_| Value::Boolean(true))
        .map_err(|e| CorvoError::file_system(e.to_string()))
}

pub fn stat(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let path = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("fs.stat requires a path"))?;

    let metadata = fs::metadata(path).map_err(|e| CorvoError::file_system(e.to_string()))?;

    let mut result = HashMap::new();
    result.insert("size".to_string(), Value::Number(metadata.len() as f64));
    result.insert("is_dir".to_string(), Value::Boolean(metadata.is_dir()));
    result.insert(
        "permissions".to_string(),
        Value::String(format!("{:?}", metadata.permissions())),
    );
    result.insert(
        "modified_at".to_string(),
        Value::Number(
            metadata
                .modified()
                .map(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as f64
                })
                .unwrap_or(0.0),
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
    fn test_write_and_read() {
        let dir = std::env::temp_dir().join("corvo_test_write");
        let path = dir.to_string_lossy().to_string();

        let _ = fs::remove_file(&path);

        let write_args = vec![
            Value::String(path.clone()),
            Value::String("hello world".to_string()),
        ];
        assert_eq!(
            write(&write_args, &empty_args()).unwrap(),
            Value::Boolean(true)
        );

        let read_args = vec![Value::String(path.clone())];
        assert_eq!(
            read(&read_args, &empty_args()).unwrap(),
            Value::String("hello world".to_string())
        );

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_read_not_found() {
        let args = vec![Value::String("/nonexistent/path/file.txt".to_string())];
        assert!(read(&args, &empty_args()).is_err());
    }

    #[test]
    fn test_exists_true() {
        let args = vec![Value::String("/tmp".to_string())];
        assert_eq!(exists(&args, &empty_args()).unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_exists_false() {
        let args = vec![Value::String("/nonexistent/path".to_string())];
        assert_eq!(exists(&args, &empty_args()).unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_mkdir_and_list_dir() {
        let dir = std::env::temp_dir().join("corvo_test_dir");
        let path = dir.to_string_lossy().to_string();
        let _ = fs::remove_dir_all(&path);

        let mkdir_args = vec![Value::String(path.clone()), Value::Boolean(true)];
        assert_eq!(
            mkdir(&mkdir_args, &empty_args()).unwrap(),
            Value::Boolean(true)
        );

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn test_write_no_args() {
        assert!(write(&[], &empty_args()).is_err());
    }

    #[test]
    fn test_exists_no_args() {
        assert!(exists(&[], &empty_args()).is_err());
    }

    #[test]
    fn test_delete_no_args() {
        assert!(delete(&[], &empty_args()).is_err());
    }

    #[test]
    fn test_stat_directory() {
        let args = vec![Value::String("/tmp".to_string())];
        let result = stat(&args, &empty_args()).unwrap();
        match result {
            Value::Map(m) => {
                assert!(m.contains_key("size"));
                assert!(m.contains_key("is_dir"));
            }
            _ => panic!("Expected Map"),
        }
    }
}
