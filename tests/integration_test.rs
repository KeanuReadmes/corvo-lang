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
fn test_list_new() {
    let state = run_with_state(
        r#"
        var.set("items", list.new())
        var.set("empty_check", list.is_empty(var.get("items")))
        var.set("items", list.push(var.get("items"), "first"))
        var.set("count", list.len(var.get("items")))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("empty_check").unwrap(),
        corvo_lang::type_system::Value::Boolean(true)
    );
    assert_eq!(
        state.var_get("count").unwrap(),
        corvo_lang::type_system::Value::Number(1.0)
    );
}

#[test]
fn test_map_new() {
    let state = run_with_state(
        r#"
        var.set("data", map.new())
        var.set("empty_check", map.is_empty(var.get("data")))
        var.set("data", map.set(var.get("data"), "key", "value"))
        var.set("count", map.len(var.get("data")))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("empty_check").unwrap(),
        corvo_lang::type_system::Value::Boolean(true)
    );
    assert_eq!(
        state.var_get("count").unwrap(),
        corvo_lang::type_system::Value::Number(1.0)
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

// --- Index-based shorthand assignment (@var["key"] = val, @var[idx] = val) ---

#[test]
fn test_at_map_index_set() {
    let state = run_with_state(
        r#"
        var.set("person", {"name": "Alice", "city": "London"})
        @person["city"] = "Tokyo"
        var.set("city", @person["city"])
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("city").unwrap(),
        corvo_lang::type_system::Value::String("Tokyo".to_string())
    );
}

#[test]
fn test_at_map_index_set_new_key() {
    let state = run_with_state(
        r#"
        var.set("config", {"host": "localhost"})
        @config["port"] = 8080
        var.set("port", @config["port"])
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("port").unwrap(),
        corvo_lang::type_system::Value::Number(8080.0)
    );
}

#[test]
fn test_at_list_index_set() {
    let state = run_with_state(
        r#"
        var.set("nums", [10, 20, 30])
        @nums[1] = 99
        var.set("second", @nums[1])
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("second").unwrap(),
        corvo_lang::type_system::Value::Number(99.0)
    );
}

#[test]
fn test_at_list_index_set_out_of_bounds() {
    let result = run_with_state(
        r#"
        var.set("items", [1, 2, 3])
        @items[10] = 99
        "#,
    );
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("out of bounds"));
}

#[test]
fn test_at_index_read() {
    let state = run_with_state(
        r#"
        var.set("data", {"x": 42})
        var.set("val", @data["x"])
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("val").unwrap(),
        corvo_lang::type_system::Value::Number(42.0)
    );
}

// --- New list methods ---

#[test]
fn test_list_delete() {
    let state = run_with_state(
        r#"
        var.set("items", ["a", "b", "c"])
        var.set("items", list.delete(var.get("items"), 1))
        var.set("count", list.len(var.get("items")))
        var.set("first", list.get(var.get("items"), 0))
        var.set("second", list.get(var.get("items"), 1))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("count").unwrap(),
        corvo_lang::type_system::Value::Number(2.0)
    );
    assert_eq!(
        state.var_get("first").unwrap(),
        corvo_lang::type_system::Value::String("a".to_string())
    );
    assert_eq!(
        state.var_get("second").unwrap(),
        corvo_lang::type_system::Value::String("c".to_string())
    );
}

#[test]
fn test_list_sort() {
    let state = run_with_state(
        r#"
        var.set("items", ["banana", "apple", "cherry"])
        var.set("sorted", list.sort(var.get("items")))
        var.set("first", list.get(var.get("sorted"), 0))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("first").unwrap(),
        corvo_lang::type_system::Value::String("apple".to_string())
    );
}

#[test]
fn test_list_find() {
    let state = run_with_state(
        r#"
        var.set("items", ["a", "b", "c"])
        var.set("idx", list.find(var.get("items"), "b"))
        var.set("missing", list.find(var.get("items"), "z"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("idx").unwrap(),
        corvo_lang::type_system::Value::Number(1.0)
    );
    assert_eq!(
        state.var_get("missing").unwrap(),
        corvo_lang::type_system::Value::Number(-1.0)
    );
}

#[test]
fn test_list_slice() {
    let state = run_with_state(
        r#"
        var.set("items", [1, 2, 3, 4, 5])
        var.set("sliced", list.slice(var.get("items"), 1, 4))
        var.set("count", list.len(var.get("sliced")))
        var.set("first", list.get(var.get("sliced"), 0))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("count").unwrap(),
        corvo_lang::type_system::Value::Number(3.0)
    );
    assert_eq!(
        state.var_get("first").unwrap(),
        corvo_lang::type_system::Value::Number(2.0)
    );
}

#[test]
fn test_list_unique() {
    let state = run_with_state(
        r#"
        var.set("items", ["a", "b", "a", "c", "b"])
        var.set("unique", list.unique(var.get("items")))
        var.set("count", list.len(var.get("unique")))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("count").unwrap(),
        corvo_lang::type_system::Value::Number(3.0)
    );
}

#[test]
fn test_list_flatten() {
    let state = run_with_state(
        r#"
        var.set("nested", [[1, 2], [3, 4], [5]])
        var.set("flat", list.flatten(var.get("nested")))
        var.set("count", list.len(var.get("flat")))
        var.set("first", list.get(var.get("flat"), 0))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("count").unwrap(),
        corvo_lang::type_system::Value::Number(5.0)
    );
    assert_eq!(
        state.var_get("first").unwrap(),
        corvo_lang::type_system::Value::Number(1.0)
    );
}

// --- New map methods ---

#[test]
fn test_map_delete() {
    let state = run_with_state(
        r#"
        var.set("person", {"name": "Alice", "age": 30, "city": "Tokyo"})
        var.set("person", map.delete(var.get("person"), "age"))
        var.set("has_age", map.has_key(var.get("person"), "age"))
        var.set("count", map.len(var.get("person")))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("has_age").unwrap(),
        corvo_lang::type_system::Value::Boolean(false)
    );
    assert_eq!(
        state.var_get("count").unwrap(),
        corvo_lang::type_system::Value::Number(2.0)
    );
}

#[test]
fn test_map_has() {
    let state = run_with_state(
        r#"
        var.set("data", {"key": "value"})
        var.set("found", map.has(var.get("data"), "key"))
        var.set("missing", map.has(var.get("data"), "other"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("found").unwrap(),
        corvo_lang::type_system::Value::Boolean(true)
    );
    assert_eq!(
        state.var_get("missing").unwrap(),
        corvo_lang::type_system::Value::Boolean(false)
    );
}

#[test]
fn test_map_entries() {
    let state = run_with_state(
        r#"
        var.set("data", {"b": 2, "a": 1})
        var.set("entries", map.entries(var.get("data")))
        var.set("count", list.len(var.get("entries")))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("count").unwrap(),
        corvo_lang::type_system::Value::Number(2.0)
    );
    // entries should be sorted by key
    let entries = state.var_get("entries").unwrap();
    if let corvo_lang::type_system::Value::List(list) = entries {
        if let corvo_lang::type_system::Value::Map(first) = &list[0] {
            assert_eq!(
                first.get("key").unwrap(),
                &corvo_lang::type_system::Value::String("a".to_string())
            );
        } else {
            panic!("Expected map entry");
        }
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_static_vs_var_independence() {
    let state = run_with_state(
        r#"
        prep {
            static.set("x", 2)
        }
        var.set("x", 1)
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

// --- Prep Block Integration Tests ---

#[test]
fn test_prep_block_sets_static() {
    let state = run_with_state(
        r#"
        prep {
            static.set("api_version", "v2")
        }
        var.set("result", static.get("api_version"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("v2".to_string())
    );
}

#[test]
fn test_prep_block_vars_not_available_outside() {
    let result = run_with_state(
        r#"
        prep {
            var.set("temp", 42)
        }
        var.set("x", var.get("temp"))
        "#,
    );
    // var "temp" should not be available outside the prep block
    assert!(result.is_err());
}

#[test]
fn test_prep_block_must_come_first() {
    let result = run_with_state(
        r#"
        var.set("x", 1)
        prep {
            static.set("config", "value")
        }
        "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{}", err).contains("prep block must come before all other statements"));
}

#[test]
fn test_static_set_outside_prep_is_error() {
    let result = run_with_state(r#"static.set("x", 1)"#);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{}", err).contains("static.set() can only be used inside a prep block"));
}

#[test]
fn test_prep_block_multiple_statics() {
    let state = run_with_state(
        r#"
        prep {
            static.set("host", "localhost")
            static.set("port", 8080)
            static.set("debug", true)
        }
        var.set("h", static.get("host"))
        var.set("p", static.get("port"))
        var.set("d", static.get("debug"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("h").unwrap(),
        corvo_lang::type_system::Value::String("localhost".to_string())
    );
    assert_eq!(
        state.var_get("p").unwrap(),
        corvo_lang::type_system::Value::Number(8080.0)
    );
    assert_eq!(
        state.var_get("d").unwrap(),
        corvo_lang::type_system::Value::Boolean(true)
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
        var.set("encrypted", crypto.encrypt("mykey", "secret"))
        var.set("decrypted", crypto.decrypt("mykey", var.get("encrypted")))
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
fn test_crypto_hash_file() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_str().unwrap().to_string();
    std::fs::write(&path, "hello").unwrap();

    let source = format!(
        r#"var.set("h", crypto.hash_file("sha256", "{path}"))"#,
        path = path
    );
    let state = run_with_state(&source).unwrap();
    match state.var_get("h").unwrap() {
        corvo_lang::type_system::Value::String(h) => assert_eq!(h.len(), 64),
        _ => panic!("Expected String"),
    }
}

#[test]
fn test_crypto_hash_file_matches_hash() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_str().unwrap().to_string();
    std::fs::write(&path, "hello").unwrap();

    let source = format!(
        r#"
        var.set("file_hash", crypto.hash_file("md5", "{path}"))
        var.set("data_hash", crypto.hash("md5", "hello"))
        "#,
        path = path
    );
    let state = run_with_state(&source).unwrap();
    assert_eq!(
        state.var_get("file_hash").unwrap(),
        state.var_get("data_hash").unwrap()
    );
}

#[test]
fn test_crypto_checksum() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_str().unwrap().to_string();
    std::fs::write(&path, "checksum content").unwrap();

    let source = format!(
        r#"
        var.set("cs", crypto.checksum("{path}"))
        var.set("len", string.len(var.get("cs")))
        "#,
        path = path
    );
    let state = run_with_state(&source).unwrap();
    assert_eq!(
        state.var_get("len").unwrap(),
        corvo_lang::type_system::Value::Number(64.0)
    );
}

#[test]
fn test_crypto_checksum_matches_hash_file_sha256() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_str().unwrap().to_string();
    std::fs::write(&path, "verify content").unwrap();

    let source = format!(
        r#"
        var.set("cs", crypto.checksum("{path}"))
        var.set("hf", crypto.hash_file("sha256", "{path}"))
        "#,
        path = path
    );
    let state = run_with_state(&source).unwrap();
    assert_eq!(state.var_get("cs").unwrap(), state.var_get("hf").unwrap());
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

// --- @ Variable Shortcut Tests ---

#[test]
fn test_at_var_set_shortcut() {
    let state = run_with_state(
        r#"
        @name = "Corvo"
        @count = 42
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("name").unwrap(),
        corvo_lang::type_system::Value::String("Corvo".to_string())
    );
    assert_eq!(
        state.var_get("count").unwrap(),
        corvo_lang::type_system::Value::Number(42.0)
    );
}

#[test]
fn test_at_var_get_shortcut() {
    let state = run_with_state(
        r#"
        @greeting = "hello"
        @result = string.to_upper(@greeting)
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("HELLO".to_string())
    );
}

#[test]
fn test_at_var_shortcut_in_expression() {
    let state = run_with_state(
        r#"
        @a = 10
        @b = 20
        @sum = math.add(@a, @b)
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("sum").unwrap(),
        corvo_lang::type_system::Value::Number(30.0)
    );
}

#[test]
fn test_at_var_shortcut_interop_with_var_get_set() {
    let state = run_with_state(
        r#"
        var.set("x", 100)
        @y = math.add(@x, 1)
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("y").unwrap(),
        corvo_lang::type_system::Value::Number(101.0)
    );
}

#[test]
fn test_at_var_get_in_string_interpolation() {
    let state = run_with_state(
        r#"
        @name = "World"
        @msg = "Hello ${@name}!"
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("msg").unwrap(),
        corvo_lang::type_system::Value::String("Hello World!".to_string())
    );
}

// --- dont_panic Block Tests ---

#[test]
fn test_dont_panic_suppresses_var_not_found() {
    // Without dont_panic this would error; with it, it should succeed silently
    let state = run_with_state(
        r#"
        @x = 1
        dont_panic {
            sys.echo(@non_existent)
        }
        @x = math.add(@x, 1)
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("x").unwrap(),
        corvo_lang::type_system::Value::Number(2.0)
    );
}

#[test]
fn test_dont_panic_allows_normal_execution() {
    let state = run_with_state(
        r#"
        @result = "initial"
        dont_panic {
            @result = "updated"
        }
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("updated".to_string())
    );
}

#[test]
fn test_var_not_found_panics_outside_dont_panic() {
    let result = run_with_state(r#"sys.echo(@non_existent)"#);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("non_existent"));
}

#[test]
fn test_dont_panic_suppresses_all_errors() {
    // dont_panic catches any runtime error, not just variable-not-found
    let state = run_with_state(
        r#"
        @flag = "ok"
        dont_panic {
            math.div(1, 0)
            @flag = "should not reach"
        }
        "#,
    )
    .unwrap();
    // flag stays "ok" because div-by-zero was caught before the assignment
    assert_eq!(
        state.var_get("flag").unwrap(),
        corvo_lang::type_system::Value::String("ok".to_string())
    );
}

// --- browse Block Tests ---

#[test]
fn test_browse_list_collects_values() {
    let state = run_with_state(
        r#"
        @my_list = ["one", "two", "three"]
        @last_key = 0
        @last_val = ""
        browse(@my_list, key, val) {
            @last_key = @key
            @last_val = @val
        }
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("last_key").unwrap(),
        corvo_lang::type_system::Value::Number(2.0)
    );
    assert_eq!(
        state.var_get("last_val").unwrap(),
        corvo_lang::type_system::Value::String("three".to_string())
    );
}

#[test]
fn test_browse_list_accumulates_values() {
    let state = run_with_state(
        r#"
        @nums = [10, 20, 30]
        @sum = 0
        browse(@nums, idx, num) {
            @sum = math.add(@sum, @num)
        }
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("sum").unwrap(),
        corvo_lang::type_system::Value::Number(60.0)
    );
}

#[test]
fn test_browse_list_key_is_numeric_index() {
    let state = run_with_state(
        r#"
        @items = ["a", "b", "c"]
        @key_sum = 0
        browse(@items, k, v) {
            @key_sum = math.add(@key_sum, @k)
        }
        "#,
    )
    .unwrap();
    // indices 0 + 1 + 2 = 3
    assert_eq!(
        state.var_get("key_sum").unwrap(),
        corvo_lang::type_system::Value::Number(3.0)
    );
}

#[test]
fn test_browse_map_key_and_value() {
    let state = run_with_state(
        r#"
        @my_map = {"answer": 42}
        @found_key = ""
        @found_val = 0
        browse(@my_map, prop, val) {
            @found_key = @prop
            @found_val = @val
        }
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("found_key").unwrap(),
        corvo_lang::type_system::Value::String("answer".to_string())
    );
    assert_eq!(
        state.var_get("found_val").unwrap(),
        corvo_lang::type_system::Value::Number(42.0)
    );
}

#[test]
fn test_browse_map_collects_all_keys() {
    let state = run_with_state(
        r#"
        @m = {"a": 1, "b": 2, "c": 3}
        @key_list = []
        browse(@m, k, v) {
            @key_list = list.push(@key_list, @k)
        }
        "#,
    )
    .unwrap();
    // Map keys are iterated in sorted order, so ["a", "b", "c"]
    let key_list = state.var_get("key_list").unwrap();
    if let corvo_lang::type_system::Value::List(keys) = key_list {
        let mut key_strs: Vec<String> = keys.iter().map(|v| v.to_string()).collect();
        key_strs.sort();
        assert_eq!(key_strs, vec!["a", "b", "c"]);
    } else {
        panic!("Expected a list");
    }
}

#[test]
fn test_browse_empty_list() {
    let state = run_with_state(
        r#"
        @empty = []
        @count = 0
        browse(@empty, k, v) {
            @count = math.add(@count, 1)
        }
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("count").unwrap(),
        corvo_lang::type_system::Value::Number(0.0)
    );
}

#[test]
fn test_browse_nested() {
    let state = run_with_state(
        r#"
        @outer = [["a", "b"], ["c", "d"]]
        @total = 0
        browse(@outer, i, inner) {
            browse(@inner, j, v) {
                @total = math.add(@total, 1)
            }
        }
        "#,
    )
    .unwrap();
    // 2 outer items * 2 inner items = 4 total iterations
    assert_eq!(
        state.var_get("total").unwrap(),
        corvo_lang::type_system::Value::Number(4.0)
    );
}

#[test]
fn test_browse_with_terminate() {
    let state = run_with_state(
        r#"
        @items = [1, 2, 3, 4, 5]
        @sum = 0
        browse(@items, k, v) {
            @sum = math.add(@sum, @v)
            try {
                assert_eq(@v, 3)
                terminate
            } fallback {}
        }
        "#,
    )
    .unwrap();
    // Should sum 1+2+3=6 then terminate
    assert_eq!(
        state.var_get("sum").unwrap(),
        corvo_lang::type_system::Value::Number(6.0)
    );
}

#[test]
fn test_browse_type_error_on_non_collection() {
    let result = run_with_state(
        r#"
        @x = "not a list"
        browse(@x, k, v) {}
        "#,
    );
    assert!(result.is_err());
}

#[test]
fn test_env_parse_basic() {
    let state = run_with_state(
        r#"
        var.set("env_str", "HOST=localhost\nPORT=8080")
        var.set("config", env.parse(var.get("env_str")))
        var.set("host", map.get(var.get("config"), "HOST"))
        var.set("port", map.get(var.get("config"), "PORT"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("host").unwrap(),
        corvo_lang::type_system::Value::String("localhost".to_string())
    );
    assert_eq!(
        state.var_get("port").unwrap(),
        corvo_lang::type_system::Value::String("8080".to_string())
    );
}

#[test]
fn test_env_parse_skips_comments() {
    // This test verifies env.parse returns a map with only the non-comment entries.
    // Comment-skipping with '#' is fully exercised by unit tests in standard_lib::env.
    let state = run_with_state(
        r#"
        var.set("line1", "NAME=corvo")
        var.set("line2", "\nTAG=v1")
        var.set("env_str", string.concat(var.get("line1"), var.get("line2")))
        var.set("config", env.parse(var.get("env_str")))
        var.set("name", map.get(var.get("config"), "NAME"))
        var.set("tag", map.get(var.get("config"), "TAG"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("name").unwrap(),
        corvo_lang::type_system::Value::String("corvo".to_string())
    );
    assert_eq!(
        state.var_get("tag").unwrap(),
        corvo_lang::type_system::Value::String("v1".to_string())
    );
}

// ---------------------------------------------------------------------------
// notifications module tests
// ---------------------------------------------------------------------------

#[test]
fn test_notifications_smtp_requires_all_args() {
    // Missing args should produce an error
    let result = run_with_state(r#"notifications.smtp("smtp.example.com")"#);
    assert!(result.is_err());
}

#[test]
fn test_notifications_slack_requires_args() {
    let result = run_with_state(r#"notifications.slack()"#);
    assert!(result.is_err());
}

#[test]
fn test_notifications_slack_requires_message() {
    let result = run_with_state(r#"notifications.slack("https://hooks.slack.com/x")"#);
    assert!(result.is_err());
}

#[test]
fn test_notifications_telegram_requires_args() {
    let result = run_with_state(r#"notifications.telegram("token")"#);
    assert!(result.is_err());
}

#[test]
fn test_notifications_mattermost_requires_args() {
    let result = run_with_state(r#"notifications.mattermost()"#);
    assert!(result.is_err());
}

#[test]
fn test_notifications_gitter_requires_args() {
    let result = run_with_state(r#"notifications.gitter("token")"#);
    assert!(result.is_err());
}

#[test]
fn test_notifications_messenger_requires_args() {
    let result = run_with_state(r#"notifications.messenger("token")"#);
    assert!(result.is_err());
}

#[test]
fn test_notifications_discord_requires_args() {
    let result = run_with_state(r#"notifications.discord()"#);
    assert!(result.is_err());
}

#[test]
fn test_notifications_teams_requires_args() {
    let result = run_with_state(r#"notifications.teams()"#);
    assert!(result.is_err());
}

#[test]
fn test_notifications_x_requires_all_args() {
    let result = run_with_state(r#"notifications.x("key", "secret")"#);
    assert!(result.is_err());
}

#[test]
fn test_notifications_os_requires_args() {
    let result = run_with_state(r#"notifications.os()"#);
    assert!(result.is_err());
}

#[test]
fn test_notifications_os_returns_success_map() {
    // notifications.os runs a best-effort system command; the result map must
    // always contain a "success" key regardless of whether the platform has a
    // notification daemon available.
    let state = run_with_state(
        r#"
        var.set("res", notifications.os("Test", "Integration test notification"))
        var.set("has_success", map.has_key(var.get("res"), "success"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("has_success").unwrap(),
        corvo_lang::type_system::Value::Boolean(true)
    );
}

#[test]
fn test_notifications_smtp_bad_from_address() {
    // An invalid from address should be rejected
    let result = run_with_state(
        r#"
        notifications.smtp(
            "smtp.example.com", 587,
            "user", "pass",
            "not-an-email",
            "to@example.com",
            "subject", "body"
        )
        "#,
    );
    assert!(result.is_err());
}

#[test]
fn test_notifications_smtp_bad_to_address() {
    // An invalid to address should be rejected
    let result = run_with_state(
        r#"
        notifications.smtp(
            "smtp.example.com", 587,
            "user", "pass",
            "from@example.com",
            "not-an-email",
            "subject", "body"
        )
        "#,
    );
    assert!(result.is_err());
}

#[test]
fn test_notifications_lint_all_functions_known() {
    use corvo_lang::diagnostic::KNOWN_FUNCTIONS;
    let notification_fns = [
        "notifications.smtp",
        "notifications.slack",
        "notifications.telegram",
        "notifications.mattermost",
        "notifications.gitter",
        "notifications.messenger",
        "notifications.discord",
        "notifications.teams",
        "notifications.x",
        "notifications.os",
        "notifications.irc",
    ];
    for f in &notification_fns {
        assert!(
            KNOWN_FUNCTIONS.contains(f),
            "{} missing from KNOWN_FUNCTIONS",
            f
        );
    }
}

#[test]
fn test_notifications_irc_requires_host() {
    let result = run_with_state(r#"notifications.irc()"#);
    assert!(result.is_err());
}

#[test]
fn test_notifications_irc_requires_port() {
    let result = run_with_state(r#"notifications.irc("irc.example.com")"#);
    assert!(result.is_err());
}

#[test]
fn test_notifications_irc_requires_nickname() {
    let result = run_with_state(r#"notifications.irc("irc.example.com", 6667)"#);
    assert!(result.is_err());
}

#[test]
fn test_notifications_irc_requires_channel() {
    let result = run_with_state(r#"notifications.irc("irc.example.com", 6667, "nick")"#);
    assert!(result.is_err());
}

#[test]
fn test_notifications_irc_requires_message() {
    let result = run_with_state(r##"notifications.irc("irc.example.com", 6667, "nick", "#test")"##);
    assert!(result.is_err());
}

#[test]
fn test_notifications_irc_unreachable_host_returns_error() {
    // Port 1 on loopback is almost certainly not open, so connect fails immediately.
    let result =
        run_with_state(r##"notifications.irc("127.0.0.1", 1, "corvo-bot", "#test", "hello")"##);
    assert!(result.is_err());
}

// --- Match Expression Tests ---

#[test]
fn test_match_string_literal() {
    let state = run_with_state(
        r#"
        var.set("file", "hosts")
        var.set("result", match(var.get("file")) {
            "" => "empty",
            "hosts" => "hosts file",
            _ => "not empty"
        })
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("hosts file".to_string())
    );
}

#[test]
fn test_match_wildcard() {
    let state = run_with_state(
        r#"
        var.set("file", "something")
        var.set("result", match(var.get("file")) {
            "" => "empty",
            "hosts" => "hosts file",
            _ => "not empty"
        })
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("not empty".to_string())
    );
}

#[test]
fn test_match_empty_string() {
    let state = run_with_state(
        r#"
        var.set("file", "")
        var.set("result", match(var.get("file")) {
            "" => "empty",
            "hosts" => "hosts file",
            _ => "not empty"
        })
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("empty".to_string())
    );
}

#[test]
fn test_match_number_pattern() {
    let state = run_with_state(
        r#"
        var.set("code", 200)
        var.set("result", match(var.get("code")) {
            200 => "OK",
            404 => "Not Found",
            _ => "Unknown"
        })
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("OK".to_string())
    );
}

#[test]
fn test_match_boolean_pattern() {
    let state = run_with_state(
        r#"
        var.set("flag", true)
        var.set("result", match(var.get("flag")) {
            true => "yes",
            false => "no",
            _ => "unknown"
        })
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("yes".to_string())
    );
}

#[test]
fn test_match_at_shorthand_assignment() {
    let state = run_with_state(
        r#"
        var.set("file", "hosts")
        @result = match(@file) {
            "" => "empty",
            "hosts" => "hosts file",
            _ => "not empty"
        }
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("hosts file".to_string())
    );
}

#[test]
fn test_match_no_match_returns_error() {
    let result = run_with_state(
        r#"
        var.set("x", "unknown")
        var.set("result", match(var.get("x")) {
            "a" => "letter a",
            "b" => "letter b"
        })
        "#,
    );
    assert!(result.is_err());
}

#[test]
fn test_match_first_matching_arm_wins() {
    let state = run_with_state(
        r#"
        var.set("x", "a")
        var.set("result", match(var.get("x")) {
            "a" => "first",
            "a" => "second",
            _ => "wildcard"
        })
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("first".to_string())
    );
}

// ---------------------------------------------------------------------------
// Regex Tests
// ---------------------------------------------------------------------------

#[test]
fn test_regex_literal_creates_value() {
    let state = run_with_state(
        r#"
        @re = /[0-9]+/
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("re").unwrap(),
        corvo_lang::type_system::Value::Regex("[0-9]+".to_string(), "".to_string())
    );
}

#[test]
fn test_regex_literal_with_flags() {
    let state = run_with_state(
        r#"
        @re = /hello/gi
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("re").unwrap(),
        corvo_lang::type_system::Value::Regex("hello".to_string(), "gi".to_string())
    );
}

#[test]
fn test_re_match_returns_true() {
    let state = run_with_state(
        r#"
        @expression = /[0-9]+/
        var.set("result", re.match(@expression, "9283"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::Boolean(true)
    );
}

#[test]
fn test_re_match_returns_false() {
    let state = run_with_state(
        r#"
        @expression = /[0-9]+/
        var.set("result", re.match(@expression, "hello"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::Boolean(false)
    );
}

#[test]
fn test_re_match_case_insensitive() {
    let state = run_with_state(
        r#"
        @re = /hello/i
        var.set("result", re.match(@re, "HELLO WORLD"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::Boolean(true)
    );
}

#[test]
fn test_re_find_returns_first_match() {
    let state = run_with_state(
        r#"
        @re = /[0-9]+/
        var.set("result", re.find(@re, "abc123def456"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("123".to_string())
    );
}

#[test]
fn test_re_find_returns_null_when_no_match() {
    let state = run_with_state(
        r#"
        @re = /[0-9]+/
        var.set("result", re.find(@re, "abcdef"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::Null
    );
}

#[test]
fn test_re_find_all_returns_all_matches() {
    let state = run_with_state(
        r#"
        @re = /[0-9]+/
        var.set("result", re.find_all(@re, "abc123def456ghi789"))
        "#,
    )
    .unwrap();
    match state.var_get("result").unwrap() {
        corvo_lang::type_system::Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert_eq!(
                items[0],
                corvo_lang::type_system::Value::String("123".to_string())
            );
            assert_eq!(
                items[1],
                corvo_lang::type_system::Value::String("456".to_string())
            );
            assert_eq!(
                items[2],
                corvo_lang::type_system::Value::String("789".to_string())
            );
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_re_replace() {
    let state = run_with_state(
        r#"
        @re = /[0-9]+/
        var.set("result", re.replace(@re, "abc123def", "NUM"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("abcNUMdef".to_string())
    );
}

#[test]
fn test_re_replace_all() {
    let state = run_with_state(
        r#"
        @re = /[0-9]+/
        var.set("result", re.replace_all(@re, "abc123def456", "NUM"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("abcNUMdefNUM".to_string())
    );
}

#[test]
fn test_re_split() {
    let state = run_with_state(
        r#"
        @re = /,\s*/
        var.set("result", re.split(@re, "a, b, c"))
        "#,
    )
    .unwrap();
    match state.var_get("result").unwrap() {
        corvo_lang::type_system::Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert_eq!(
                items[0],
                corvo_lang::type_system::Value::String("a".to_string())
            );
            assert_eq!(
                items[1],
                corvo_lang::type_system::Value::String("b".to_string())
            );
            assert_eq!(
                items[2],
                corvo_lang::type_system::Value::String("c".to_string())
            );
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_re_new_creates_regex_value() {
    let state = run_with_state(
        r#"
        var.set("result", re.new("[0-9]+", "i"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::Regex("[0-9]+".to_string(), "i".to_string())
    );
}

#[test]
fn test_method_call_style_match() {
    let state = run_with_state(
        r#"
        @expression = /[0-9]+/
        var.set("result", @expression.match("9283"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::Boolean(true)
    );
}

#[test]
fn test_method_call_style_find() {
    let state = run_with_state(
        r#"
        @re = /[0-9]+/
        var.set("result", @re.find("abc123def"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("123".to_string())
    );
}

#[test]
fn test_match_expression_with_regex_pattern() {
    let state = run_with_state(
        r#"
        @my_number = "9123"
        var.set("result", match(@my_number) {
            /[0-9]+/ => "matched"
            _ => "booo"
        })
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("matched".to_string())
    );
}

#[test]
fn test_match_expression_regex_no_match_falls_through_to_wildcard() {
    let state = run_with_state(
        r#"
        var.set("text", "hello")
        var.set("result", match(var.get("text")) {
            /[0-9]+/ => "is number"
            _ => "not a number"
        })
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("not a number".to_string())
    );
}

#[test]
fn test_match_expression_multiple_regex_patterns() {
    let state = run_with_state(
        r#"
        var.set("text", "hello@example.com")
        var.set("result", match(var.get("text")) {
            /[0-9]+/ => "number"
            /[a-z]+@[a-z]+\.[a-z]+/ => "email"
            _ => "other"
        })
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("email".to_string())
    );
}

#[test]
fn test_inline_regex_in_re_match() {
    let state = run_with_state(
        r#"
        var.set("result", re.match(/[0-9]+/, "abc123"))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::Boolean(true)
    );
}

#[test]
fn test_template_render_basic() {
    let state = run_with_state(
        r#"
        var.set("data", map.new())
        var.set("data", map.set(var.get("data"), "name", "Corvo"))
        var.set("result", template.render("Hello, {{name}}!", var.get("data")))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("Hello, Corvo!".to_string())
    );
}

#[test]
fn test_template_render_multiple_vars() {
    let state = run_with_state(
        r#"
        var.set("data", map.new())
        var.set("data", map.set(var.get("data"), "lang", "Rust"))
        var.set("data", map.set(var.get("data"), "version", "0.1.0"))
        var.set("result", template.render("{{lang}} v{{version}}", var.get("data")))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("Rust v0.1.0".to_string())
    );
}

#[test]
fn test_template_render_missing_key_empty() {
    let state = run_with_state(
        r#"
        var.set("data", map.new())
        var.set("result", template.render("Hello, {{missing}}!", var.get("data")))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("Hello, !".to_string())
    );
}

#[test]
fn test_template_render_no_placeholders() {
    let state = run_with_state(
        r#"
        var.set("data", map.new())
        var.set("result", template.render("static text", var.get("data")))
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("static text".to_string())
    );
}

#[test]
fn test_template_render_file_missing() {
    let result = run_with_state(
        r#"
        var.set("data", map.new())
        var.set("result", template.render_file("/nonexistent/template.hbs", var.get("data")))
        "#,
    );
    assert!(result.is_err());
}

// --- Slicing Tests ---

#[test]
fn test_list_slice_start_end() {
    let state = run_with_state(
        r#"
        var.set("nums", [10, 20, 30, 40, 50])
        var.set("result", @nums[1:3])
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::List(vec![
            corvo_lang::type_system::Value::Number(20.0),
            corvo_lang::type_system::Value::Number(30.0),
        ])
    );
}

#[test]
fn test_list_slice_from_start() {
    let state = run_with_state(
        r#"
        var.set("nums", [10, 20, 30, 40, 50])
        var.set("result", @nums[:2])
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::List(vec![
            corvo_lang::type_system::Value::Number(10.0),
            corvo_lang::type_system::Value::Number(20.0),
        ])
    );
}

#[test]
fn test_list_slice_to_end() {
    let state = run_with_state(
        r#"
        var.set("nums", [10, 20, 30, 40, 50])
        var.set("result", @nums[3:])
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::List(vec![
            corvo_lang::type_system::Value::Number(40.0),
            corvo_lang::type_system::Value::Number(50.0),
        ])
    );
}

#[test]
fn test_list_slice_all() {
    let state = run_with_state(
        r#"
        var.set("nums", [10, 20, 30])
        var.set("result", @nums[:])
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::List(vec![
            corvo_lang::type_system::Value::Number(10.0),
            corvo_lang::type_system::Value::Number(20.0),
            corvo_lang::type_system::Value::Number(30.0),
        ])
    );
}

#[test]
fn test_list_slice_negative_index_via_var() {
    // Negative numbers must be stored in a variable (no unary-minus literal syntax).
    let state = run_with_state(
        r#"
        var.set("nums", [10, 20, 30, 40, 50])
        var.set("neg", math.sub(0, 2))
        var.set("result", @nums[@neg:])
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::List(vec![
            corvo_lang::type_system::Value::Number(40.0),
            corvo_lang::type_system::Value::Number(50.0),
        ])
    );
}

#[test]
fn test_string_slice_start_end() {
    let state = run_with_state(
        r#"
        var.set("word", "Corvo")
        var.set("result", @word[1:4])
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("orv".to_string())
    );
}

#[test]
fn test_string_slice_from_start() {
    let state = run_with_state(
        r#"
        var.set("word", "Corvo")
        var.set("result", @word[:3])
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("Cor".to_string())
    );
}

#[test]
fn test_string_slice_to_end() {
    let state = run_with_state(
        r#"
        var.set("word", "Corvo")
        var.set("result", @word[2:])
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("rvo".to_string())
    );
}

#[test]
fn test_string_slice_all() {
    let state = run_with_state(
        r#"
        var.set("word", "hello")
        var.set("result", @word[:])
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("hello".to_string())
    );
}

#[test]
fn test_string_slice_negative_index_via_var() {
    // Negative numbers must be stored in a variable (no unary-minus literal syntax).
    let state = run_with_state(
        r#"
        var.set("word", "Corvo")
        var.set("neg", math.sub(0, 3))
        var.set("result", @word[@neg:])
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("rvo".to_string())
    );
}

#[test]
fn test_slice_inline_on_literal() {
    let state = run_with_state(
        r#"
        var.set("result", [1, 2, 3, 4, 5][1:3])
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::List(vec![
            corvo_lang::type_system::Value::Number(2.0),
            corvo_lang::type_system::Value::Number(3.0),
        ])
    );
}

#[test]
fn test_slice_in_string_interpolation() {
    let state = run_with_state(
        r#"
        var.set("word", "Corvo")
        var.set("result", "${@word[0:3]}")
        "#,
    )
    .unwrap();
    assert_eq!(
        state.var_get("result").unwrap(),
        corvo_lang::type_system::Value::String("Cor".to_string())
    );
}
