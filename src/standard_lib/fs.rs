use crate::type_system::Value;
use crate::{CorvoError, CorvoResult};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::{FileTypeExt, MetadataExt};

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

/// Metadata for a single path (same shape as elements of [`read_dir_meta`]).
pub fn read_meta(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let path_s = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("fs.read_meta requires a path"))?;

    let follow_symlinks = args.get(1).and_then(|v| v.as_bool()).unwrap_or(false);

    let path = Path::new(path_s.as_str());
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path_s.clone());

    let is_symlink_entry = fs::symlink_metadata(path)
        .map(|sm| sm.file_type().is_symlink())
        .unwrap_or(false);

    let meta = if follow_symlinks {
        fs::metadata(path)
            .or_else(|_| fs::symlink_metadata(path))
            .map_err(|e| CorvoError::file_system(e.to_string()))?
    } else {
        fs::symlink_metadata(path).map_err(|e| CorvoError::file_system(e.to_string()))?
    };
    let child_s = path_s.clone();

    let mut m: HashMap<String, Value> = HashMap::new();
    m.insert("name".to_string(), Value::String(name));
    m.insert("path".to_string(), Value::String(child_s.clone()));

    let ft = meta.file_type();
    let is_symlink = is_symlink_entry;
    let is_dir = ft.is_dir();
    let is_file = ft.is_file();

    let symlink_target = if is_symlink {
        fs::read_link(path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    } else {
        String::new()
    };

    m.insert("is_symlink".to_string(), Value::Boolean(is_symlink));
    m.insert("is_dir".to_string(), Value::Boolean(is_dir));
    m.insert("is_file".to_string(), Value::Boolean(is_file));
    m.insert("symlink_target".to_string(), Value::String(symlink_target));

    #[cfg(unix)]
    {
        let mode = meta.mode() & 0o7777;
        m.insert("mode".to_string(), Value::Number(mode as f64));
        m.insert(
            "mode_string".to_string(),
            Value::String(unix_mode_string(&meta, &ft)),
        );
        m.insert("inode".to_string(), Value::Number(meta.ino() as f64));
        m.insert("nlink".to_string(), Value::Number(meta.nlink() as f64));
        m.insert("uid".to_string(), Value::Number(meta.uid() as f64));
        m.insert("gid".to_string(), Value::Number(meta.gid() as f64));
        m.insert("blocks".to_string(), Value::Number(meta.blocks() as f64));
        m.insert(
            "user".to_string(),
            Value::String(
                uzers::get_user_by_uid(meta.uid())
                    .map(|u| u.name().to_string_lossy().to_string())
                    .unwrap_or_else(|| meta.uid().to_string()),
            ),
        );
        m.insert(
            "group".to_string(),
            Value::String(
                uzers::get_group_by_gid(meta.gid())
                    .map(|g| g.name().to_string_lossy().to_string())
                    .unwrap_or_else(|| meta.gid().to_string()),
            ),
        );
        let rdev = meta.rdev();
        m.insert("major".to_string(), Value::Number(unix_major(rdev) as f64));
        m.insert("minor".to_string(), Value::Number(unix_minor(rdev) as f64));
        m.insert(
            "file_type_char".to_string(),
            Value::String(unix_file_type_char(&ft).to_string()),
        );
    }

    #[cfg(not(unix))]
    {
        m.insert("mode".to_string(), Value::Number(0.0));
        m.insert(
            "mode_string".to_string(),
            Value::String(
                if is_dir {
                    "d?????????"
                } else if is_symlink {
                    "l?????????"
                } else {
                    "-?????????"
                }
                .to_string(),
            ),
        );
        m.insert("inode".to_string(), Value::Number(0.0));
        m.insert("nlink".to_string(), Value::Number(1.0));
        m.insert("uid".to_string(), Value::Number(0.0));
        m.insert("gid".to_string(), Value::Number(0.0));
        m.insert("blocks".to_string(), Value::Number(0.0));
        m.insert("user".to_string(), Value::String(String::new()));
        m.insert("group".to_string(), Value::String(String::new()));
        m.insert("major".to_string(), Value::Number(0.0));
        m.insert("minor".to_string(), Value::Number(0.0));
        m.insert(
            "file_type_char".to_string(),
            Value::String(
                if is_dir {
                    "d"
                } else if is_symlink {
                    "l"
                } else {
                    "-"
                }
                .to_string(),
            ),
        );
    }

    m.insert("size".to_string(), Value::Number(meta.len() as f64));
    #[cfg(unix)]
    {
        let mode = meta.mode();
        m.insert(
            "is_executable".to_string(),
            Value::Boolean(mode & 0o111 != 0),
        );
    }
    #[cfg(not(unix))]
    {
        m.insert("is_executable".to_string(), Value::Boolean(false));
    }
    push_times(&mut m, &meta);

    Ok(Value::Map(m))
}

pub fn read_link(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let path = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("fs.read_link requires a path"))?;

    let target =
        fs::read_link(path.as_str()).map_err(|e| CorvoError::file_system(e.to_string()))?;
    Ok(Value::String(target.to_string_lossy().to_string()))
}

/// Directory entries with metadata suitable for GNU `ls` (uses `lstat` per entry).
pub fn read_dir_meta(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let path = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("fs.read_dir_meta requires a path"))?;

    let follow_symlinks = args.get(1).and_then(|v| v.as_bool()).unwrap_or(false);

    let base = Path::new(path.as_str());
    let rd = fs::read_dir(base).map_err(|e| CorvoError::file_system(e.to_string()))?;

    let mut entries: Vec<Value> = Vec::new();
    for item in rd {
        let item = item.map_err(|e| CorvoError::file_system(e.to_string()))?;
        let name = item.file_name().to_string_lossy().to_string();
        let child_path: PathBuf = base.join(&name);
        let child_s = child_path.to_string_lossy().to_string();

        let entry_is_symlink = fs::symlink_metadata(&child_path)
            .map(|sm| sm.file_type().is_symlink())
            .unwrap_or(false);

        let meta = if follow_symlinks {
            fs::metadata(&child_path)
                .or_else(|_| fs::symlink_metadata(&child_path))
                .map_err(|e| CorvoError::file_system(e.to_string()))?
        } else {
            fs::symlink_metadata(&child_path).map_err(|e| CorvoError::file_system(e.to_string()))?
        };

        let mut m: HashMap<String, Value> = HashMap::new();
        m.insert("name".to_string(), Value::String(name.clone()));
        m.insert("path".to_string(), Value::String(child_s.clone()));

        let ft = meta.file_type();
        let is_symlink = entry_is_symlink;
        let is_dir = ft.is_dir();
        let is_file = ft.is_file();

        let symlink_target = if entry_is_symlink {
            fs::read_link(&child_path)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };

        m.insert("is_symlink".to_string(), Value::Boolean(is_symlink));
        m.insert("is_dir".to_string(), Value::Boolean(is_dir));
        m.insert("is_file".to_string(), Value::Boolean(is_file));
        m.insert("symlink_target".to_string(), Value::String(symlink_target));

        #[cfg(unix)]
        {
            let mode = meta.mode() & 0o7777;
            m.insert("mode".to_string(), Value::Number(mode as f64));
            m.insert(
                "mode_string".to_string(),
                Value::String(unix_mode_string(&meta, &ft)),
            );
            m.insert("inode".to_string(), Value::Number(meta.ino() as f64));
            m.insert("nlink".to_string(), Value::Number(meta.nlink() as f64));
            m.insert("uid".to_string(), Value::Number(meta.uid() as f64));
            m.insert("gid".to_string(), Value::Number(meta.gid() as f64));
            m.insert("blocks".to_string(), Value::Number(meta.blocks() as f64));
            m.insert(
                "user".to_string(),
                Value::String(
                    uzers::get_user_by_uid(meta.uid())
                        .map(|u| u.name().to_string_lossy().to_string())
                        .unwrap_or_else(|| meta.uid().to_string()),
                ),
            );
            m.insert(
                "group".to_string(),
                Value::String(
                    uzers::get_group_by_gid(meta.gid())
                        .map(|g| g.name().to_string_lossy().to_string())
                        .unwrap_or_else(|| meta.gid().to_string()),
                ),
            );

            let rdev = meta.rdev();
            m.insert("major".to_string(), Value::Number(unix_major(rdev) as f64));
            m.insert("minor".to_string(), Value::Number(unix_minor(rdev) as f64));

            m.insert(
                "file_type_char".to_string(),
                Value::String(unix_file_type_char(&ft).to_string()),
            );
        }

        #[cfg(not(unix))]
        {
            m.insert("mode".to_string(), Value::Number(0.0));
            m.insert(
                "mode_string".to_string(),
                Value::String(
                    if is_dir {
                        "d?????????"
                    } else if is_symlink {
                        "l?????????"
                    } else {
                        "-?????????"
                    }
                    .to_string(),
                ),
            );
            m.insert("inode".to_string(), Value::Number(0.0));
            m.insert("nlink".to_string(), Value::Number(1.0));
            m.insert("uid".to_string(), Value::Number(0.0));
            m.insert("gid".to_string(), Value::Number(0.0));
            m.insert("blocks".to_string(), Value::Number(0.0));
            m.insert("user".to_string(), Value::String(String::new()));
            m.insert("group".to_string(), Value::String(String::new()));
            m.insert("major".to_string(), Value::Number(0.0));
            m.insert("minor".to_string(), Value::Number(0.0));
            m.insert(
                "file_type_char".to_string(),
                Value::String(
                    if is_dir {
                        "d"
                    } else if is_symlink {
                        "l"
                    } else {
                        "-"
                    }
                    .to_string(),
                ),
            );
        }

        m.insert("size".to_string(), Value::Number(meta.len() as f64));
        #[cfg(unix)]
        {
            let mode = meta.mode();
            let ix = mode & 0o111 != 0;
            m.insert("is_executable".to_string(), Value::Boolean(ix));
        }
        #[cfg(not(unix))]
        {
            m.insert("is_executable".to_string(), Value::Boolean(false));
        }

        push_times(&mut m, &meta);

        entries.push(Value::Map(m));
    }

    Ok(Value::List(entries))
}

/// Parent directory path (empty string if none, e.g. root on Unix).
/// For `\".\"` / `\"./\"`, returns the parent of the current working directory so
/// `ls -a` can synthesize a `..` entry (Rust `Path::parent` is `None` for `.`).
pub fn path_parent(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let path_s = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("fs.path_parent requires a path"))?;
    let path_norm = path_s.trim_end_matches('/');
    if path_norm.is_empty() || path_norm == "." {
        let out = std::env::current_dir()
            .ok()
            .and_then(|c| c.parent().map(|p| p.to_string_lossy().to_string()))
            .unwrap_or_default();
        return Ok(Value::String(out));
    }
    let p = Path::new(path_s.as_str());
    let out = p
        .parent()
        .map(|x| x.to_string_lossy().to_string())
        .unwrap_or_default();
    Ok(Value::String(out))
}

/// Path of `path` relative to `base` (both strings). If `path` is not under `base`, returns `path` unchanged.
pub fn path_relative(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let base_s = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("fs.path_relative requires base path"))?;
    let path_s = args
        .get(1)
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("fs.path_relative requires path"))?;

    let base = Path::new(base_s.as_str());
    let path = Path::new(path_s.as_str());
    let rel = match path.strip_prefix(base) {
        Ok(r) => r.to_string_lossy().to_string().replace('\\', "/"),
        Err(_) => path_s.to_string(),
    };
    if rel.is_empty() {
        Ok(Value::String(".".to_string()))
    } else {
        Ok(Value::String(rel))
    }
}

#[cfg(unix)]
fn unix_major(dev: u64) -> u32 {
    ((((dev & 0xfff00) >> 8) | ((dev & 0xfffff00000000000) >> 32)) & 0xffffffff) as u32
}

#[cfg(unix)]
fn unix_minor(dev: u64) -> u32 {
    (((dev & 0xff) | ((dev >> 12) & 0xffffff00)) & 0xffffffff) as u32
}

#[cfg(unix)]
fn push_times(m: &mut HashMap<String, Value>, meta: &fs::Metadata) {
    m.insert("mtime_sec".to_string(), Value::Number(meta.mtime() as f64));
    m.insert(
        "mtime_nsec".to_string(),
        Value::Number(meta.mtime_nsec() as f64),
    );
    m.insert("atime_sec".to_string(), Value::Number(meta.atime() as f64));
    m.insert(
        "atime_nsec".to_string(),
        Value::Number(meta.atime_nsec() as f64),
    );
    m.insert("ctime_sec".to_string(), Value::Number(meta.ctime() as f64));
    m.insert(
        "ctime_nsec".to_string(),
        Value::Number(meta.ctime_nsec() as f64),
    );
}

#[cfg(not(unix))]
fn push_times(m: &mut HashMap<String, Value>, meta: &fs::Metadata) {
    fn split(t: Result<std::time::SystemTime, std::time::SystemTimeError>) -> (f64, f64) {
        match t.and_then(|s| s.duration_since(std::time::UNIX_EPOCH)) {
            Ok(d) => (d.as_secs() as f64, d.subsec_nanos() as f64),
            Err(_) => (0.0, 0.0),
        }
    }

    let (mts, mtn) = split(meta.modified());
    let (ats, atn) = split(meta.accessed());
    let (cts, ctn) = split(meta.created());

    m.insert("mtime_sec".to_string(), Value::Number(mts));
    m.insert("mtime_nsec".to_string(), Value::Number(mtn));
    m.insert("atime_sec".to_string(), Value::Number(ats));
    m.insert("atime_nsec".to_string(), Value::Number(atn));
    m.insert("ctime_sec".to_string(), Value::Number(cts));
    m.insert("ctime_nsec".to_string(), Value::Number(ctn));
}

#[cfg(unix)]
fn unix_file_type_char(ft: &fs::FileType) -> char {
    if ft.is_symlink() {
        'l'
    } else if ft.is_dir() {
        'd'
    } else if ft.is_file() {
        '-'
    } else if ft.is_fifo() {
        'p'
    } else if ft.is_socket() {
        's'
    } else if ft.is_block_device() {
        'b'
    } else if ft.is_char_device() {
        'c'
    } else {
        '?'
    }
}

#[cfg(unix)]
fn unix_mode_string(meta: &fs::Metadata, ft: &fs::FileType) -> String {
    let mode = meta.mode();
    let mut s = String::with_capacity(10);
    s.push(unix_file_type_char(ft));
    let r = |m: u32, bit: u32| if m & bit != 0 { 'r' } else { '-' };
    let w = |m: u32, bit: u32| if m & bit != 0 { 'w' } else { '-' };
    let xb = |m: u32, bit: u32| m & bit != 0;

    let ur = r(mode, 0o400);
    let uw = w(mode, 0o200);
    let ux = xb(mode, 0o100);

    let gr = r(mode, 0o040);
    let gw = w(mode, 0o020);
    let gx = xb(mode, 0o010);

    let or = r(mode, 0o004);
    let ow = w(mode, 0o002);
    let ox = xb(mode, 0o001);

    s.push(ur);
    s.push(uw);
    s.push(if mode & 0o4000 != 0 {
        if ux {
            's'
        } else {
            'S'
        }
    } else if ux {
        'x'
    } else {
        '-'
    });

    s.push(gr);
    s.push(gw);
    s.push(if mode & 0o2000 != 0 {
        if gx {
            's'
        } else {
            'S'
        }
    } else if gx {
        'x'
    } else {
        '-'
    });

    s.push(or);
    s.push(ow);
    s.push(if mode & 0o1000 != 0 {
        if ox {
            't'
        } else {
            'T'
        }
    } else if ox {
        'x'
    } else {
        '-'
    });

    s
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

    #[test]
    fn test_path_parent_dot_is_parent_of_cwd() {
        let expected = std::env::current_dir()
            .ok()
            .and_then(|c| c.parent().map(|p| p.to_string_lossy().to_string()))
            .unwrap_or_default();
        let args = vec![Value::String(".".to_string())];
        assert_eq!(
            path_parent(&args, &empty_args()).unwrap(),
            Value::String(expected)
        );
    }
}
