use corvo_lang::compiler::Evaluator;
use corvo_lang::lexer::Lexer;
use corvo_lang::parser::Parser;
use corvo_lang::{CorvoResult, RuntimeState};

fn run_with_state(source: &str) -> CorvoResult<RuntimeState> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;
    let mut state = RuntimeState::new();
    let mut evaluator = Evaluator::new();
    evaluator.run(&program, &mut state)?;
    Ok(state)
}

// --- End-to-End Programs ---

#[test]
fn test_hello_world() {
    let state = run_with_state(r#"sys.echo("Hello, World!")"#).unwrap();
    // echo returns Null, just verify no error
    assert!(state.is_empty());
}

#[test]
fn test_variable_arithmetic() {
    let state = run_with_state(
        r#"
        var.set("a", 10)
        var.set("b", 20)
        var.set("sum", math.add(var.get("a"), var.get("b")))
        var.set("product", math.mul(var.get("a"), var.get("b")))
        var.set("quotient", math.div(var.get("b"), var.get("a")))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("sum").unwrap(),
        corvo_lang::type_system::Value::Number(30.0)
    );
    assert_eq!(
        state.var_get("product").unwrap(),
        corvo_lang::type_system::Value::Number(200.0)
    );
    assert_eq!(
        state.var_get("quotient").unwrap(),
        corvo_lang::type_system::Value::Number(2.0)
    );
}

#[test]
fn test_string_operations() {
    let state = run_with_state(
        r#"
        var.set("greeting", string.concat("Hello", " World"))
        var.set("upper", string.to_upper(var.get("greeting")))
        var.set("lower", string.to_lower("HELLO"))
        var.set("len", string.len(var.get("greeting")))
        var.set("trimmed", string.trim("  spaces  "))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("greeting").unwrap(),
        corvo_lang::type_system::Value::String("Hello World".to_string())
    );
    assert_eq!(
        state.var_get("upper").unwrap(),
        corvo_lang::type_system::Value::String("HELLO WORLD".to_string())
    );
    assert_eq!(
        state.var_get("lower").unwrap(),
        corvo_lang::type_system::Value::String("hello".to_string())
    );
    assert_eq!(
        state.var_get("len").unwrap(),
        corvo_lang::type_system::Value::Number(11.0)
    );
    assert_eq!(
        state.var_get("trimmed").unwrap(),
        corvo_lang::type_system::Value::String("spaces".to_string())
    );
}

#[test]
fn test_list_operations() {
    let state = run_with_state(
        r#"
        var.set("items", [])
        var.set("items", list.push(var.get("items"), "a"))
        var.set("items", list.push(var.get("items"), "b"))
        var.set("items", list.push(var.get("items"), "c"))
        var.set("first", list.get(var.get("items"), 0))
        var.set("last", list.get(var.get("items"), 2))
        var.set("count", list.len(var.get("items")))
        var.set("joined", list.join(var.get("items"), ", "))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("first").unwrap(),
        corvo_lang::type_system::Value::String("a".to_string())
    );
    assert_eq!(
        state.var_get("last").unwrap(),
        corvo_lang::type_system::Value::String("c".to_string())
    );
    assert_eq!(
        state.var_get("count").unwrap(),
        corvo_lang::type_system::Value::Number(3.0)
    );
    assert_eq!(
        state.var_get("joined").unwrap(),
        corvo_lang::type_system::Value::String("a, b, c".to_string())
    );
}

#[test]
fn test_map_operations() {
    let state = run_with_state(
        r#"
        var.set("config", {})
        var.set("config", map.set(var.get("config"), "name", "corvo"))
        var.set("config", map.set(var.get("config"), "version", "0.1.0"))
        var.set("name", map.get(var.get("config"), "name"))
        var.set("has_name", map.has_key(var.get("config"), "name"))
        var.set("key_count", map.len(var.get("config")))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("name").unwrap(),
        corvo_lang::type_system::Value::String("corvo".to_string())
    );
    assert_eq!(
        state.var_get("has_name").unwrap(),
        corvo_lang::type_system::Value::Boolean(true)
    );
    assert_eq!(
        state.var_get("key_count").unwrap(),
        corvo_lang::type_system::Value::Number(2.0)
    );
}

#[test]
fn test_try_fallback_scenarios() {
    // Success case - no fallback
    let state = run_with_state(
        r#"
        var.set("result", "init")
        try {
            assert_eq(1, 1)
            var.set("result", "success")
        } fallback {
            var.set("result", "failed")
        }
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("success".to_string())
    );
}

#[test]
fn test_try_fallback_failure() {
    // Failure case - fallback runs
    let state = run_with_state(
        r#"
        var.set("result", "init")
        try {
            assert_eq(1, 2)
            var.set("result", "success")
        } fallback {
            var.set("result", "failed")
        }
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("failed".to_string())
    );
}

#[test]
fn test_try_multiple_fallbacks() {
    let state = run_with_state(
        r#"
        var.set("result", "init")
        try {
            assert_eq(1, 2)
        } fallback {
            assert_eq(3, 4)
        } fallback {
            var.set("result", "third time's the charm")
        }
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("third time's the charm".to_string())
    );
}

#[test]
fn test_loop_with_counter() {
    let state = run_with_state(
        r#"
        var.set("counter", 0)
        loop {
            var.set("counter", math.add(var.get("counter"), 1))
            try {
                assert_eq(var.get("counter"), 5)
                terminate
            } fallback {
            }
        }
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("counter").unwrap(),
        corvo_lang::type_system::Value::Number(5.0)
    );
}

#[test]
fn test_string_interpolation_complex() {
    let state = run_with_state(
        r#"
        var.set("name", "Corvo")
        var.set("version", "0.1.0")
        var.set("msg", "${var.get("name")} v${var.get("version")}")
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("msg").unwrap(),
        corvo_lang::type_system::Value::String("Corvo v0.1.0".to_string())
    );
}

#[test]
fn test_nested_expressions() {
    let state = run_with_state(
        r#"
        var.set("result", math.add(math.mul(3, 4), math.div(10, 2)))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::Number(17.0)
    );
}

#[test]
fn test_list_literal_and_methods() {
    let state = run_with_state(
        r#"
        var.set("nums", [1, 2, 3])
        var.set("count", list.len(var.get("nums")))
        var.set("first", list.get(var.get("nums"), 0))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("count").unwrap(),
        corvo_lang::type_system::Value::Number(3.0)
    );
    assert_eq!(
        state.var_get("first").unwrap(),
        corvo_lang::type_system::Value::Number(1.0)
    );
}

#[test]
fn test_map_literal_and_methods() {
    let state = run_with_state(
        r#"
        var.set("person", {"name": "Alice", "age": 30})
        var.set("name", map.get(var.get("person"), "name"))
        var.set("has_age", map.has_key(var.get("person"), "age"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("name").unwrap(),
        corvo_lang::type_system::Value::String("Alice".to_string())
    );
    assert_eq!(
        state.var_get("has_age").unwrap(),
        corvo_lang::type_system::Value::Boolean(true)
    );
}

#[test]
fn test_static_vs_var_independence() {
    let state = run_with_state(
        r#"
        var.set("x", 1)
        static.set("x", 2)
        var.set("var_x", var.get("x"))
        var.set("static_x", static.get("x"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("var_x").unwrap(),
        corvo_lang::type_system::Value::Number(1.0)
    );
    assert_eq!(
        state.var_get("static_x").unwrap(),
        corvo_lang::type_system::Value::Number(2.0)
    );
}

// --- Error Handling Integration Tests ---

#[test]
fn test_parse_error() {
    let result = run_with_state("var.set x 42");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{}", err).contains("Expected '('"));
}

#[test]
fn test_runtime_error_var_not_found() {
    let result = run_with_state(r#"var.set("x", var.get("nonexistent"))"#);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("nonexistent"));
}

#[test]
fn test_runtime_error_division_by_zero() {
    let result = run_with_state("math.div(1, 0)");
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("zero"));
}

#[test]
fn test_runtime_error_unknown_function() {
    let result = run_with_state("not_a_function()");
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("not_a_function"));
}

#[test]
fn test_runtime_error_index_out_of_bounds() {
    let result = run_with_state("list.get([], 0)");
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("out of bounds"));
}

#[test]
fn test_assertion_error_messages() {
    let result = run_with_state("assert_eq(1, 2)");
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("1 != 2"));
}

// --- Comprehensive Program Tests ---

#[test]
fn test_factorial_computation() {
    let state = run_with_state(
        r#"
        var.set("n", 5)
        var.set("result", 1)
        var.set("i", 1)
        loop {
            var.set("result", math.mul(var.get("result"), var.get("i")))
            var.set("i", math.add(var.get("i"), 1))
            try {
                assert_eq(var.get("i"), math.add(var.get("n"), 1))
                terminate
            } fallback {
            }
        }
        "#,
    )
    .unwrap();
    // 5! = 120
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::Number(120.0)
    );
}

#[test]
fn test_fibonacci_computation() {
    let state = run_with_state(
        r#"
        var.set("a", 0)
        var.set("b", 1)
        var.set("n", 10)
        var.set("i", 0)
        loop {
            var.set("temp", var.get("b"))
            var.set("b", math.add(var.get("a"), var.get("b")))
            var.set("a", var.get("temp"))
            var.set("i", math.add(var.get("i"), 1))
            try {
                assert_eq(var.get("i"), var.get("n"))
                terminate
            } fallback {
            }
        }
        "#,
    )
    .unwrap();
    // fib(11) = 89 (10 iterations starting from 0,1)
    assert_eq!(
        state.var_get("b").unwrap(),
        corvo_lang::type_system::Value::Number(89.0)
    );
}

#[test]
fn test_accumulator_pattern() {
    let state = run_with_state(
        r#"
        var.set("sum", 0)
        var.set("i", 1)
        loop {
            var.set("sum", math.add(var.get("sum"), var.get("i")))
            var.set("i", math.add(var.get("i"), 1))
            try {
                assert_eq(var.get("i"), 101)
                terminate
            } fallback {
            }
        }
        "#,
    )
    .unwrap();
    // sum(1..100) = 5050
    assert_eq!(
        state.var_get("sum").unwrap(),
        corvo_lang::type_system::Value::Number(5050.0)
    );
}

#[test]
fn test_json_roundtrip() {
    let state = run_with_state(
        r#"
        var.set("data", json.parse("{\"name\": \"test\", \"count\": 42}"))
        var.set("name", map.get(var.get("data"), "name"))
        var.set("count", map.get(var.get("data"), "count"))
        var.set("json_out", json.stringify(var.get("data")))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("name").unwrap(),
        corvo_lang::type_system::Value::String("test".to_string())
    );
    assert_eq!(
        state.var_get("count").unwrap(),
        corvo_lang::type_system::Value::Number(42.0)
    );
}

#[test]
fn test_crypto_hash() {
    let state = run_with_state(
        r#"
        var.set("hash", crypto.hash("md5", "hello"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("hash").unwrap(),
        corvo_lang::type_system::Value::String("5d41402abc4b2a76b9719d911017c592".to_string())
    );
}

#[test]
fn test_crypto_encrypt_decrypt() {
    let state = run_with_state(
        r#"
        var.set("encrypted", crypto.encrypt("secret", "mykey"))
        var.set("decrypted", crypto.decrypt(var.get("encrypted"), "mykey"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("decrypted").unwrap(),
        corvo_lang::type_system::Value::String("secret".to_string())
    );
}

#[test]
fn test_uuid_generation() {
    let state = run_with_state(
        r#"
        var.set("id", crypto.uuid())
        var.set("len", string.len(var.get("id")))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("len").unwrap(),
        corvo_lang::type_system::Value::Number(36.0)
    );
}

#[test]
fn test_file_write_read_exists() {
    let state = run_with_state(
        r#"
        var.set("path", "/tmp/corvo_integration_test.txt")
        fs.write(var.get("path"), "integration test content")
        var.set("exists", fs.exists(var.get("path")))
        var.set("content", fs.read(var.get("path")))
        fs.delete(var.get("path"))
        var.set("deleted", fs.exists(var.get("path")))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("exists").unwrap(),
        corvo_lang::type_system::Value::Boolean(true)
    );
    assert_eq!(
        state.var_get("content").unwrap(),
        corvo_lang::type_system::Value::String("integration test content".to_string())
    );
    assert_eq!(
        state.var_get("deleted").unwrap(),
        corvo_lang::type_system::Value::Boolean(false)
    );
}

#[test]
fn test_os_info() {
    let state = run_with_state(
        r#"
        var.set("info", os.info())
        var.set("os_name", map.get(var.get("info"), "os"))
        "#,
    )
    .unwrap();
    let os_name = state.var_get("os_name").unwrap();
    assert!(matches!(os_name, corvo_lang::type_system::Value::String(_)));
}

#[test]
fn test_os_env() {
    std::env::set_var("CORVO_INTEGRATION_TEST", "test_value");
    let state = run_with_state(
        r#"
        var.set("val", os.get_env("CORVO_INTEGRATION_TEST", "default"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("val").unwrap(),
        corvo_lang::type_system::Value::String("test_value".to_string())
    );
    std::env::remove_var("CORVO_INTEGRATION_TEST");
}

// --- run_source / run_file API Tests ---

#[test]
fn test_run_source_api() {
    let result = corvo_lang::run_source(r#"var.set("x", 42)"#);
    assert!(result.is_ok());
}

#[test]
fn test_run_source_error() {
    let result = corvo_lang::run_source("invalid syntax here");
    assert!(result.is_err());
}
