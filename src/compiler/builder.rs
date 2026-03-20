use crate::runtime::RuntimeState;
use crate::type_system::Value;
use crate::CorvoError;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct Compiler {
    source: String,
    _source_path: PathBuf,
    build_mode: BuildMode,
    statics: HashMap<String, Value>,
}

pub enum BuildMode {
    Debug,
    Release,
}

impl Compiler {
    pub fn new(source: String, source_path: PathBuf) -> Self {
        Self {
            source,
            _source_path: source_path,
            build_mode: BuildMode::Release,
            statics: HashMap::new(),
        }
    }

    pub fn with_debug(mut self) -> Self {
        self.build_mode = BuildMode::Debug;
        self
    }

    /// Pre-executes the script to capture static variable values.
    /// This is the key step: static.set("key", os.get_env("VAR")) runs at
    /// compile time, and the resulting value is baked into the binary.
    pub fn pre_execute(&mut self) -> Result<(), CorvoError> {
        let mut lexer = crate::lexer::Lexer::new(&self.source);
        let tokens = lexer.tokenize()?;

        let mut parser = crate::parser::Parser::new(tokens);
        let program = parser.parse()?;

        let mut state = RuntimeState::new();
        let mut evaluator = crate::compiler::Evaluator::new();
        // Run the script. Runtime errors are OK - we just want static values.
        let _ = evaluator.run(&program, &mut state);

        self.statics = state.statics_snapshot();
        Ok(())
    }

    pub fn static_count(&self) -> usize {
        self.statics.len()
    }

    pub fn compile(&self, output: &Path) -> Result<PathBuf, CorvoError> {
        let crate_root = find_crate_root()?;
        let build_dir = create_build_dir()?;

        self.generate_cargo_toml(&build_dir, &crate_root)?;
        self.generate_main_rs(&build_dir)?;

        let binary = self.run_cargo_build(&build_dir)?;
        let final_binary = copy_binary(&binary, output)?;

        Ok(final_binary)
    }

    fn generate_cargo_toml(&self, build_dir: &Path, crate_root: &Path) -> Result<(), CorvoError> {
        let crate_root_str = crate_root
            .to_str()
            .ok_or_else(|| CorvoError::io("Invalid crate root path".to_string()))?
            .replace('\\', "/");

        let cargo_toml = format!(
            "[package]\n\
             name = \"corvo_compiled\"\n\
             version = \"0.1.0\"\n\
             edition = \"2021\"\n\
             \n\
             [dependencies]\n\
             corvo-lang = {{ path = \"{}\" }}\n\
             \n\
             [profile.release]\n\
             opt-level = 2\n\
             lto = false\n",
            crate_root_str
        );

        std::fs::write(build_dir.join("Cargo.toml"), cargo_toml)
            .map_err(|e| CorvoError::io(format!("Failed to write Cargo.toml: {}", e)))?;

        Ok(())
    }

    fn generate_main_rs(&self, build_dir: &Path) -> Result<(), CorvoError> {
        let src_dir = build_dir.join("src");
        std::fs::create_dir_all(&src_dir)
            .map_err(|e| CorvoError::io(format!("Failed to create src dir: {}", e)))?;

        let escaped_source = escape_for_rust(&self.source);

        let mut main_rs = String::new();
        main_rs.push_str("fn main() {\n");

        // Generate static variable initialization from pre-computed values
        main_rs.push_str("    let mut state = corvo_lang::RuntimeState::new();\n");
        for (key, value) in &self.statics {
            let key_lit = escape_for_rust(key);
            let value_code = value_to_rust_code(value);
            main_rs.push_str(&format!(
                "    state.static_set(\"{}\".to_string(), {});\n",
                key_lit, value_code
            ));
        }

        // Generate and execute the script
        main_rs.push_str(&format!(
            "    let source = String::from(\"{}\");\n",
            escaped_source
        ));
        main_rs.push_str("    match corvo_lang::run_source_with_state(&source, &mut state) {\n");
        main_rs.push_str("        Ok(_) => std::process::exit(0),\n");
        main_rs.push_str("        Err(e) => {\n");
        main_rs.push_str("            eprintln!(\"{}\", e);\n");
        main_rs.push_str("            std::process::exit(e.exit_code());\n");
        main_rs.push_str("        }\n");
        main_rs.push_str("    }\n");
        main_rs.push_str("}\n");

        std::fs::write(src_dir.join("main.rs"), main_rs)
            .map_err(|e| CorvoError::io(format!("Failed to write main.rs: {}", e)))?;

        Ok(())
    }

    fn run_cargo_build(&self, build_dir: &Path) -> Result<PathBuf, CorvoError> {
        let profile = match self.build_mode {
            BuildMode::Debug => "debug",
            BuildMode::Release => "release",
        };

        let mut cmd = std::process::Command::new("cargo");
        cmd.arg("build");
        if matches!(self.build_mode, BuildMode::Release) {
            cmd.arg("--release");
        }
        cmd.current_dir(build_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let output = cmd
            .output()
            .map_err(|e| CorvoError::io(format!("Failed to run cargo: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CorvoError::io(format!("Compilation failed:\n{}", stderr)));
        }

        let binary_name = if cfg!(target_os = "windows") {
            "corvo_compiled.exe"
        } else {
            "corvo_compiled"
        };

        let binary = build_dir.join("target").join(profile).join(binary_name);

        if !binary.exists() {
            return Err(CorvoError::io("Compiled binary not found".to_string()));
        }

        Ok(binary)
    }
}

fn find_crate_root() -> Result<PathBuf, CorvoError> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_path = PathBuf::from(manifest_dir);
    if manifest_path.join("Cargo.toml").exists() {
        return Ok(manifest_path);
    }

    let exe = std::env::current_exe()
        .map_err(|e| CorvoError::io(format!("Cannot find executable: {}", e)))?;

    let mut current = exe
        .parent()
        .ok_or_else(|| CorvoError::io("Cannot find parent of executable".to_string()))?;

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                if content.contains("corvo-lang") || content.contains("corvo_lang") {
                    return Ok(current.to_path_buf());
                }
            }
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    Err(CorvoError::io(
        "Cannot find corvo-lang crate root. Run from within the corvo-lang project directory."
            .to_string(),
    ))
}

fn create_build_dir() -> Result<PathBuf, CorvoError> {
    let base = std::env::temp_dir().join("corvo_build");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base)
        .map_err(|e| CorvoError::io(format!("Failed to create build dir: {}", e)))?;
    Ok(base)
}

fn copy_binary(source: &Path, target: &Path) -> Result<PathBuf, CorvoError> {
    let final_target = if target.is_dir() {
        let name = if cfg!(target_os = "windows") {
            "out.exe"
        } else {
            "out"
        };
        target.join(name)
    } else {
        target.to_path_buf()
    };

    std::fs::copy(source, &final_target)
        .map_err(|e| CorvoError::io(format!("Failed to copy binary: {}", e)))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&final_target)
            .map_err(|e| CorvoError::io(format!("Failed to read permissions: {}", e)))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&final_target, perms)
            .map_err(|e| CorvoError::io(format!("Failed to set permissions: {}", e)))?;
    }

    Ok(final_target)
}

fn escape_for_rust(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);
    for ch in s.chars() {
        match ch {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(ch),
        }
    }
    result
}

fn value_to_rust_code(value: &Value) -> String {
    match value {
        Value::String(s) => format!(
            "corvo_lang::type_system::Value::String(\"{}\".to_string())",
            escape_for_rust(s)
        ),
        Value::Number(n) => {
            if n.is_infinite() && n.is_sign_positive() {
                "corvo_lang::type_system::Value::Number(f64::INFINITY)".to_string()
            } else if n.is_infinite() && n.is_sign_negative() {
                "corvo_lang::type_system::Value::Number(f64::NEG_INFINITY)".to_string()
            } else if n.is_nan() {
                "corvo_lang::type_system::Value::Number(f64::NAN)".to_string()
            } else {
                format!("corvo_lang::type_system::Value::Number({:?})", n)
            }
        }
        Value::Boolean(b) => format!("corvo_lang::type_system::Value::Boolean({})", b),
        Value::Null => "corvo_lang::type_system::Value::Null".to_string(),
        Value::List(items) => {
            let items_code: Vec<String> = items.iter().map(value_to_rust_code).collect();
            format!(
                "corvo_lang::type_system::Value::List(vec![{}])",
                items_code.join(", ")
            )
        }
        Value::Map(map) => {
            let mut entries = Vec::new();
            for (k, v) in map {
                entries.push(format!(
                    "(\"{}\".to_string(), {})",
                    escape_for_rust(k),
                    value_to_rust_code(v)
                ));
            }
            format!(
                "corvo_lang::type_system::Value::Map(std::collections::HashMap::from([{}]))",
                entries.join(", ")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_for_rust() {
        assert_eq!(escape_for_rust("hello"), "hello");
        assert_eq!(escape_for_rust("say \"hi\""), "say \\\"hi\\\"");
        assert_eq!(escape_for_rust("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn test_value_to_rust_code_string() {
        assert_eq!(
            value_to_rust_code(&Value::String("hello".to_string())),
            "corvo_lang::type_system::Value::String(\"hello\".to_string())"
        );
    }

    #[test]
    fn test_value_to_rust_code_number() {
        let code = value_to_rust_code(&Value::Number(42.0));
        assert!(code.contains("42"));
        assert!(code.contains("Value::Number"));
    }

    #[test]
    fn test_value_to_rust_code_boolean() {
        assert_eq!(
            value_to_rust_code(&Value::Boolean(true)),
            "corvo_lang::type_system::Value::Boolean(true)"
        );
    }

    #[test]
    fn test_value_to_rust_code_null() {
        assert_eq!(
            value_to_rust_code(&Value::Null),
            "corvo_lang::type_system::Value::Null"
        );
    }

    #[test]
    fn test_value_to_rust_code_list() {
        let code = value_to_rust_code(&Value::List(vec![Value::Number(1.0), Value::Number(2.0)]));
        assert!(code.contains("Value::List"));
        assert!(code.contains("vec!["));
    }

    #[test]
    fn test_value_to_rust_code_map() {
        let mut map = std::collections::HashMap::new();
        map.insert("key".to_string(), Value::Number(42.0));
        let code = value_to_rust_code(&Value::Map(map));
        assert!(code.contains("Value::Map"));
        assert!(code.contains("key"));
    }

    #[test]
    fn test_pre_execute_captures_statics() {
        std::env::set_var("CORVO_TEST_VAR", "test_value_123");
        let source = r#"prep { static.set("config", os.get_env("CORVO_TEST_VAR")) }"#.to_string();
        let mut compiler = Compiler::new(source, PathBuf::from("test.corvo"));
        compiler.pre_execute().unwrap();

        assert!(compiler.statics.contains_key("config"));
        assert_eq!(
            compiler.statics.get("config").unwrap(),
            &Value::String("test_value_123".to_string())
        );
        std::env::remove_var("CORVO_TEST_VAR");
    }

    #[test]
    fn test_pre_execute_multiple_statics() {
        std::env::set_var("CORVO_A", "value_a");
        std::env::set_var("CORVO_B", "value_b");
        let source = r#"
            prep {
                static.set("a", os.get_env("CORVO_A"))
                static.set("b", os.get_env("CORVO_B"))
                static.set("c", 42)
            }
        "#
        .to_string();
        let mut compiler = Compiler::new(source, PathBuf::from("test.corvo"));
        compiler.pre_execute().unwrap();

        assert_eq!(compiler.statics.len(), 3);
        assert_eq!(
            compiler.statics.get("a").unwrap(),
            &Value::String("value_a".to_string())
        );
        assert_eq!(
            compiler.statics.get("b").unwrap(),
            &Value::String("value_b".to_string())
        );
        assert_eq!(compiler.statics.get("c").unwrap(), &Value::Number(42.0));
        std::env::remove_var("CORVO_A");
        std::env::remove_var("CORVO_B");
    }

    #[test]
    fn test_generate_main_rs_with_statics() {
        let mut compiler = Compiler::new(
            "sys.echo(static.get(\"key\"))".to_string(),
            PathBuf::from("test.corvo"),
        );
        compiler
            .statics
            .insert("key".to_string(), Value::String("baked_value".to_string()));

        let build_dir = std::env::temp_dir().join("corvo_test_statics_gen");
        let _ = std::fs::remove_dir_all(&build_dir);
        std::fs::create_dir_all(build_dir.join("src")).unwrap();

        compiler.generate_main_rs(&build_dir).unwrap();

        let main_rs = std::fs::read_to_string(build_dir.join("src/main.rs")).unwrap();
        assert!(main_rs.contains("state.static_set"));
        assert!(main_rs.contains("baked_value"));
        assert!(main_rs.contains("run_source_with_state"));

        let _ = std::fs::remove_dir_all(&build_dir);
    }
}
