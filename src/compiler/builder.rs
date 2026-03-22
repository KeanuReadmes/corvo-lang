use crate::lexer::token::{Token, TokenType};
use crate::runtime::RuntimeState;
use crate::type_system::Value;
use crate::CorvoError;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct Compiler {
    source: String,
    /// Source with the prep block stripped out. Used when embedding source into
    /// the compiled binary: the prep block is already evaluated at compile time
    /// and its static values are baked in, so there is no need to re-run it at
    /// runtime (doing so would re-execute side effects like `fs.read` even
    /// after the file has been removed).
    source_without_prep: String,
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
        // source_without_prep starts as a copy of the full source and is
        // replaced with the stripped version once pre_execute() runs.
        let source_without_prep = source.clone();
        Self {
            source,
            source_without_prep,
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
    ///
    /// This method also strips the prep block from `source_without_prep` so
    /// that the compiled binary does not embed the prep block source code.
    /// Because the static values are already baked in, there is no need to
    /// re-run the prep block at runtime, and doing so could trigger side
    /// effects (e.g. `fs.read`) that fail if the referenced files no longer
    /// exist.
    pub fn pre_execute(&mut self) -> Result<(), CorvoError> {
        let mut lexer = crate::lexer::Lexer::new(&self.source);
        let tokens = lexer.tokenize()?;

        // Strip the prep block from the source that will be embedded in the
        // binary. The statics are baked in separately, so the prep block must
        // not run again at runtime.
        self.source_without_prep = strip_prep_block(&self.source, &tokens);

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

        let escaped_source = escape_for_rust(&self.source_without_prep);

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

/// Remove the `prep { ... }` block from `source` so that the compiled binary
/// does not embed it.  The token list (already produced by the lexer) is used
/// to locate the exact character range of the block, which means the removal
/// is accurate regardless of whitespace, comments, or string contents.
///
/// If the source contains no prep block the original source is returned
/// unchanged.
fn strip_prep_block(source: &str, tokens: &[Token]) -> String {
    // Locate the `prep` keyword token.
    let prep_idx = match tokens
        .iter()
        .position(|t| matches!(t.token_type, TokenType::Prep))
    {
        Some(idx) => idx,
        None => return source.to_string(),
    };

    let prep_start = tokens[prep_idx].span.start.offset;

    // Walk forward from the `prep` token and track brace depth to find the
    // matching closing `}`.  StringInterpolation tokens already have their
    // inner braces consumed by the lexer, so only top-level LeftBrace /
    // RightBrace tokens affect the depth counter.
    let mut brace_depth: usize = 0;
    let mut prep_end: Option<usize> = None;

    for token in &tokens[prep_idx..] {
        match &token.token_type {
            TokenType::LeftBrace => brace_depth += 1,
            TokenType::RightBrace => {
                // Guard against malformed source where `}` appears before any `{`.
                if brace_depth == 0 {
                    return source.to_string();
                }
                brace_depth -= 1;
                if brace_depth == 0 {
                    // span.end.offset points to the character *after* the `}`.
                    prep_end = Some(token.span.end.offset);
                    break;
                }
            }
            _ => {}
        }
    }

    let prep_end = match prep_end {
        Some(end) => end,
        // Malformed source – return as-is so the compiler can report the error.
        None => return source.to_string(),
    };

    // Reconstruct the source without the prep block.  Offsets are character
    // (not byte) indices, matching how the lexer advances its position.
    let chars: Vec<char> = source.chars().collect();
    let before: String = chars[..prep_start].iter().collect();
    let after: String = chars[prep_end..].iter().collect();

    format!("{}{}", before, after).trim().to_string()
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

    // --- strip_prep_block tests ---

    fn tokenize(source: &str) -> Vec<Token> {
        crate::lexer::Lexer::new(source).tokenize().unwrap()
    }

    #[test]
    fn test_strip_prep_block_removes_prep() {
        let source = "prep {\n    static.set(\"key\", 42)\n}\nsys.echo(\"hello\")";
        let tokens = tokenize(source);
        let stripped = strip_prep_block(source, &tokens);
        assert!(
            !stripped.contains("prep"),
            "stripped source must not contain 'prep'"
        );
        assert!(
            stripped.contains("sys.echo"),
            "non-prep statements must be preserved"
        );
    }

    #[test]
    fn test_strip_prep_block_no_prep() {
        let source = "sys.echo(\"hello\")";
        let tokens = tokenize(source);
        let stripped = strip_prep_block(source, &tokens);
        assert_eq!(stripped, source.trim());
    }

    #[test]
    fn test_strip_prep_block_only_prep() {
        let source = "prep {\n    static.set(\"x\", 1)\n}";
        let tokens = tokenize(source);
        let stripped = strip_prep_block(source, &tokens);
        assert!(
            stripped.is_empty(),
            "stripping a prep-only source yields an empty string"
        );
    }

    #[test]
    fn test_strip_prep_block_multiline() {
        let source = "prep {\n    static.set(\"a\", 1)\n    static.set(\"b\", 2)\n}\nvar.set(\"x\", static.get(\"a\"))";
        let tokens = tokenize(source);
        let stripped = strip_prep_block(source, &tokens);
        assert!(!stripped.contains("prep"), "prep block must be stripped");
        assert!(stripped.contains("var.set"), "non-prep code must survive");
    }

    #[test]
    fn test_pre_execute_strips_prep_block() {
        let source = "prep {\n    static.set(\"msg\", \"hello\")\n}\nsys.echo(static.get(\"msg\"))"
            .to_string();
        let mut compiler = Compiler::new(source, PathBuf::from("test.corvo"));
        compiler.pre_execute().unwrap();

        // The embedded source must not contain the prep block.
        assert!(
            !compiler.source_without_prep.contains("prep"),
            "source_without_prep must not contain 'prep'"
        );
        // The rest of the script must still be present.
        assert!(
            compiler.source_without_prep.contains("sys.echo"),
            "non-prep statements must be present in source_without_prep"
        );
        // The static value must still have been captured.
        assert_eq!(
            compiler.statics.get("msg").unwrap(),
            &Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_generate_main_rs_does_not_embed_prep_block() {
        let source = "prep {\n    static.set(\"key\", \"baked\")\n}\nsys.echo(static.get(\"key\"))"
            .to_string();
        let mut compiler = Compiler::new(source, PathBuf::from("test.corvo"));
        compiler.pre_execute().unwrap();

        let build_dir = std::env::temp_dir().join("corvo_test_no_prep_in_binary");
        let _ = std::fs::remove_dir_all(&build_dir);
        std::fs::create_dir_all(build_dir.join("src")).unwrap();

        compiler.generate_main_rs(&build_dir).unwrap();

        let main_rs = std::fs::read_to_string(build_dir.join("src/main.rs")).unwrap();
        // The prep block must not appear in the embedded source string.
        assert!(
            !main_rs.contains("static.set"),
            "prep block (static.set) must not be embedded in the binary source"
        );
        // The baked static value must appear via state.static_set.
        assert!(
            main_rs.contains("state.static_set"),
            "baked statics must be present"
        );
        assert!(main_rs.contains("baked"), "baked value must appear");
        // The rest of the corvo script must still be embedded.
        assert!(
            main_rs.contains("sys.echo"),
            "non-prep code must be embedded"
        );

        let _ = std::fs::remove_dir_all(&build_dir);
    }
}
