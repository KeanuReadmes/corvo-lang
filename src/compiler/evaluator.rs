use crate::ast::{AssertKind, Expr, Program, Stmt};
use crate::runtime::RuntimeState;
use crate::standard_lib;
use crate::type_system::Value;
use crate::{CorvoError, CorvoResult};

#[derive(Debug)]
pub enum ControlFlow {
    Continue,
    Break,
    Terminate,
}

pub struct Evaluator {
    terminate_requested: bool,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            terminate_requested: false,
        }
    }

    pub fn run(&mut self, program: &Program, state: &mut RuntimeState) -> CorvoResult<()> {
        for stmt in &program.statements {
            self.exec_stmt(stmt, state)?;
            if self.terminate_requested {
                break;
            }
        }
        Ok(())
    }

    fn exec_stmt(&mut self, stmt: &Stmt, state: &mut RuntimeState) -> CorvoResult<()> {
        match stmt {
            Stmt::StaticSet { name, value } => {
                // Skip if already set (baked in from compilation)
                if state.has_static(name) {
                    return Ok(());
                }
                let val = self.eval_expr(value, state)?;
                state.static_set(name.clone(), val);
                Ok(())
            }
            Stmt::VarSet { name, value } => {
                let val = self.eval_expr(value, state)?;
                state.var_set(name.clone(), val);
                Ok(())
            }
            Stmt::ExprStmt { expr } => {
                self.eval_expr(expr, state)?;
                Ok(())
            }
            Stmt::TryBlock { body, fallbacks } => {
                let result = self.execute_block(body, state);

                if result.is_err() {
                    for fallback in fallbacks {
                        if self.execute_block(&fallback.body, state).is_ok() {
                            return Ok(());
                        }
                    }
                }
                result
            }
            Stmt::Loop { body } => {
                while !self.terminate_requested {
                    if let Err(e) = self.execute_block(body, state) {
                        match e {
                            CorvoError::Runtime { .. } => continue,
                            _ => return Err(e),
                        }
                    }
                }
                self.terminate_requested = false;
                Ok(())
            }
            Stmt::Terminate => {
                self.terminate_requested = true;
                Ok(())
            }
            Stmt::Assert { kind, args } => self.eval_assertion(kind, args, state),
            Stmt::DontPanic { body } => {
                // Intentionally suppress all runtime errors from the block body.
                // This includes VariableNotFound, DivisionByZero, Assertion failures,
                // and any other execution error that would normally propagate.
                let _ = self.execute_block(body, state);
                Ok(())
            }
        }
    }

    fn execute_block(&mut self, body: &[Stmt], state: &mut RuntimeState) -> Result<(), CorvoError> {
        for stmt in body {
            self.exec_stmt(stmt, state)?;
            if self.terminate_requested {
                return Ok(());
            }
        }
        Ok(())
    }

    fn eval_expr(&self, expr: &Expr, state: &RuntimeState) -> CorvoResult<Value> {
        match expr {
            Expr::Literal { value } => Ok(value.clone()),
            Expr::VarGet { name } => state.var_get(name),
            Expr::StaticGet { name } => state.static_get(name),
            Expr::StringInterpolation { parts } => {
                let mut result = String::new();
                for part in parts {
                    let val = self.eval_expr(part, state)?;
                    result.push_str(&val.to_string());
                }
                Ok(Value::String(result))
            }
            Expr::FunctionCall {
                name,
                args,
                named_args,
            } => self.call_function(name, args, named_args, state),
            Expr::IndexAccess { target, index } => {
                let target_val = self.eval_expr(target, state)?;
                let index_val = self.eval_expr(index, state)?;
                self.index_access(&target_val, &index_val)
            }
        }
    }

    fn call_function(
        &self,
        name: &str,
        args: &[Expr],
        named_args: &std::collections::HashMap<String, Expr>,
        state: &RuntimeState,
    ) -> CorvoResult<Value> {
        let evaluated_args: Vec<Value> = args
            .iter()
            .map(|arg| self.eval_expr(arg, state))
            .collect::<CorvoResult<Vec<_>>>()?;

        let evaluated_named: std::collections::HashMap<String, Value> = named_args
            .iter()
            .map(|(k, v)| Ok((k.clone(), self.eval_expr(v, state)?)))
            .collect::<CorvoResult<_>>()?;

        if name == "__list__" {
            return Ok(Value::List(evaluated_args));
        }

        if name == "__map__" {
            let mut map = std::collections::HashMap::new();
            let mut i = 0;
            while i + 1 < evaluated_args.len() {
                let key = evaluated_args[i].to_string();
                let value = evaluated_args[i + 1].clone();
                map.insert(key, value);
                i += 2;
            }
            return Ok(Value::Map(map));
        }

        standard_lib::call(name, &evaluated_args, &evaluated_named, state)
    }

    fn index_access(&self, target: &Value, index: &Value) -> CorvoResult<Value> {
        match (target, index) {
            (Value::List(list), Value::Number(idx)) => {
                let idx = *idx as usize;
                list.get(idx)
                    .cloned()
                    .ok_or_else(|| CorvoError::runtime(format!("Index {} out of bounds", idx)))
            }
            (Value::Map(map), Value::String(key)) => map
                .get(key)
                .cloned()
                .ok_or_else(|| CorvoError::runtime(format!("Key '{}' not found", key))),
            _ => Err(CorvoError::r#type("Cannot index into this type")),
        }
    }

    fn eval_assertion(
        &self,
        kind: &AssertKind,
        args: &[Expr],
        state: &RuntimeState,
    ) -> CorvoResult<()> {
        if args.is_empty() {
            return Err(CorvoError::parsing(
                "Assertion requires at least one argument",
            ));
        }

        let values: Vec<Value> = args
            .iter()
            .map(|arg| self.eval_expr(arg, state))
            .collect::<CorvoResult<Vec<_>>>()?;

        match kind {
            AssertKind::Eq => {
                if values.len() != 2 {
                    return Err(CorvoError::parsing(
                        "assert_eq requires exactly 2 arguments",
                    ));
                }
                if values[0] != values[1] {
                    return Err(CorvoError::assertion(format!(
                        "{} != {}",
                        values[0], values[1]
                    )));
                }
            }
            AssertKind::Neq => {
                if values.len() != 2 {
                    return Err(CorvoError::parsing(
                        "assert_neq requires exactly 2 arguments",
                    ));
                }
                if values[0] == values[1] {
                    return Err(CorvoError::assertion(format!(
                        "{} == {}",
                        values[0], values[1]
                    )));
                }
            }
            AssertKind::Gt => {
                if values.len() != 2 {
                    return Err(CorvoError::parsing(
                        "assert_gt requires exactly 2 arguments",
                    ));
                }
                let a = values[0]
                    .as_number()
                    .ok_or_else(|| CorvoError::r#type("assert_gt requires numbers"))?;
                let b = values[1]
                    .as_number()
                    .ok_or_else(|| CorvoError::r#type("assert_gt requires numbers"))?;
                if a <= b {
                    return Err(CorvoError::assertion(format!("{} !> {}", a, b)));
                }
            }
            AssertKind::Lt => {
                if values.len() != 2 {
                    return Err(CorvoError::parsing(
                        "assert_lt requires exactly 2 arguments",
                    ));
                }
                let a = values[0]
                    .as_number()
                    .ok_or_else(|| CorvoError::r#type("assert_lt requires numbers"))?;
                let b = values[1]
                    .as_number()
                    .ok_or_else(|| CorvoError::r#type("assert_lt requires numbers"))?;
                if a >= b {
                    return Err(CorvoError::assertion(format!("{} !< {}", a, b)));
                }
            }
            AssertKind::Match => {
                if values.len() != 2 {
                    return Err(CorvoError::parsing(
                        "assert_match requires exactly 2 arguments",
                    ));
                }
                let pattern = values[0]
                    .as_string()
                    .ok_or_else(|| CorvoError::r#type("assert_match requires strings"))?;
                let target = values[1]
                    .as_string()
                    .ok_or_else(|| CorvoError::r#type("assert_match requires strings"))?;
                let re =
                    regex::Regex::new(pattern).map_err(|e| CorvoError::parsing(e.to_string()))?;
                if !re.is_match(target) {
                    return Err(CorvoError::assertion(format!(
                        "'{}' does not match '{}'",
                        target, pattern
                    )));
                }
            }
        }
        Ok(())
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn eval_source(source: &str) -> CorvoResult<RuntimeState> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;

        let mut state = RuntimeState::new();
        let mut evaluator = Evaluator::new();
        evaluator.run(&program, &mut state)?;
        Ok(state)
    }

    fn eval_expect_err(source: &str) -> CorvoError {
        eval_source(source).expect_err(&format!("Expected error for: {}", source))
    }

    // --- Basic Literals ---

    #[test]
    fn test_eval_var_set_and_get() {
        let state = eval_source(r#"var.set("x", 42)"#).unwrap();
        assert_eq!(state.var_get("x").unwrap(), Value::Number(42.0));
    }

    #[test]
    fn test_eval_static_set_and_get() {
        let state = eval_source(r#"static.set("pi", 3.14)"#).unwrap();
        assert_eq!(state.static_get("pi").unwrap(), Value::Number(3.14));
    }

    #[test]
    fn test_eval_string_literal() {
        let state = eval_source(r#"var.set("msg", "hello")"#).unwrap();
        assert_eq!(
            state.var_get("msg").unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_eval_boolean_literal() {
        let state = eval_source(r#"var.set("flag", true)"#).unwrap();
        assert_eq!(state.var_get("flag").unwrap(), Value::Boolean(true));
    }

    // --- Math Operations ---

    #[test]
    fn test_eval_math_add() {
        let state = eval_source(r#"var.set("result", math.add(1, 2))"#).unwrap();
        assert_eq!(state.var_get("result").unwrap(), Value::Number(3.0));
    }

    #[test]
    fn test_eval_math_sub() {
        let state = eval_source(r#"var.set("result", math.sub(10, 3))"#).unwrap();
        assert_eq!(state.var_get("result").unwrap(), Value::Number(7.0));
    }

    #[test]
    fn test_eval_math_mul() {
        let state = eval_source(r#"var.set("result", math.mul(4, 5))"#).unwrap();
        assert_eq!(state.var_get("result").unwrap(), Value::Number(20.0));
    }

    #[test]
    fn test_eval_math_div() {
        let state = eval_source(r#"var.set("result", math.div(10, 2))"#).unwrap();
        assert_eq!(state.var_get("result").unwrap(), Value::Number(5.0));
    }

    #[test]
    fn test_eval_division_by_zero() {
        let result = eval_source(r#"var.set("result", math.div(1, 0))"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_math_modulo() {
        let state = eval_source(r#"var.set("result", math.mod(10, 3))"#).unwrap();
        assert_eq!(state.var_get("result").unwrap(), Value::Number(1.0));
    }

    // --- String Operations ---

    #[test]
    fn test_eval_string_concat() {
        let state = eval_source(r#"var.set("result", string.concat("hello", " world"))"#).unwrap();
        assert_eq!(
            state.var_get("result").unwrap(),
            Value::String("hello world".to_string())
        );
    }

    #[test]
    fn test_eval_string_interpolation() {
        let state = eval_source(
            r#"
            var.set("name", "world")
            var.set("msg", "Hello ${var.get("name")}")
            "#,
        )
        .unwrap();
        assert_eq!(
            state.var_get("msg").unwrap(),
            Value::String("Hello world".to_string())
        );
    }

    #[test]
    fn test_eval_string_interpolation_number() {
        let state = eval_source(
            r#"
            var.set("count", 42)
            var.set("msg", "Count: ${var.get("count")}")
            "#,
        )
        .unwrap();
        assert_eq!(
            state.var_get("msg").unwrap(),
            Value::String("Count: 42".to_string())
        );
    }

    #[test]
    fn test_eval_string_interpolation_expr() {
        let state = eval_source(
            r#"
            var.set("a", 10)
            var.set("b", 20)
            var.set("msg", "Sum: ${math.add(var.get("a"), var.get("b"))}")
            "#,
        )
        .unwrap();
        assert_eq!(
            state.var_get("msg").unwrap(),
            Value::String("Sum: 30".to_string())
        );
    }

    #[test]
    fn test_eval_string_interpolation_multiple() {
        let state = eval_source(
            r#"
            var.set("first", "John")
            var.set("last", "Doe")
            var.set("msg", "${var.get("first")} ${var.get("last")}")
            "#,
        )
        .unwrap();
        assert_eq!(
            state.var_get("msg").unwrap(),
            Value::String("John Doe".to_string())
        );
    }

    // --- List Operations ---

    #[test]
    fn test_eval_list_push() {
        let state = eval_source(
            r#"
            var.set("a", 1)
            var.set("b", 2)
            var.set("items", list.push(list.push([], var.get("a")), var.get("b")))
            "#,
        )
        .unwrap();
        match state.var_get("items").unwrap() {
            Value::List(items) => assert_eq!(items.len(), 2),
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_eval_index_access_list() {
        let state = eval_source(
            r#"
            var.set("items", list.push(list.push([], "a"), "b"))
            var.set("item", list.get(var.get("items"), 1))
            "#,
        )
        .unwrap();
        assert_eq!(
            state.var_get("item").unwrap(),
            Value::String("b".to_string())
        );
    }

    #[test]
    fn test_eval_list_literal() {
        let state = eval_source(r#"var.set("items", [1, 2, 3])"#).unwrap();
        match state.var_get("items").unwrap() {
            Value::List(items) => assert_eq!(items.len(), 3),
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_eval_empty_list_literal() {
        let state = eval_source(r#"var.set("items", [])"#).unwrap();
        match state.var_get("items").unwrap() {
            Value::List(items) => assert!(items.is_empty()),
            _ => panic!("Expected List"),
        }
    }

    // --- Map Operations ---

    #[test]
    fn test_eval_map_literal() {
        let state = eval_source(r#"var.set("m", {"a": 1, "b": 2})"#).unwrap();
        match state.var_get("m").unwrap() {
            Value::Map(m) => assert_eq!(m.len(), 2),
            _ => panic!("Expected Map"),
        }
    }

    #[test]
    fn test_eval_empty_map_literal() {
        let state = eval_source(r#"var.set("m", {})"#).unwrap();
        match state.var_get("m").unwrap() {
            Value::Map(m) => assert!(m.is_empty()),
            _ => panic!("Expected Map"),
        }
    }

    // --- Control Flow ---

    #[test]
    fn test_eval_multiple_statements() {
        let state = eval_source(
            r#"
            var.set("x", 1)
            var.set("y", 2)
            var.set("sum", math.add(var.get("x"), var.get("y")))
            "#,
        )
        .unwrap();
        assert_eq!(state.var_get("sum").unwrap(), Value::Number(3.0));
    }

    #[test]
    fn test_eval_try_success() {
        let state = eval_source(
            r#"
            var.set("result", "not run")
            try {
                assert_eq(1, 1)
                var.set("result", "success")
            } fallback {
                var.set("result", "fallback")
            }
            "#,
        )
        .unwrap();
        assert_eq!(
            state.var_get("result").unwrap(),
            Value::String("success".to_string())
        );
    }

    #[test]
    fn test_eval_try_fallback() {
        let state = eval_source(
            r#"
            var.set("result", "not run")
            try {
                assert_eq(1, 2)
                var.set("result", "success")
            } fallback {
                var.set("result", "fallback")
            }
            "#,
        )
        .unwrap();
        assert_eq!(
            state.var_get("result").unwrap(),
            Value::String("fallback".to_string())
        );
    }

    #[test]
    fn test_eval_try_multiple_fallbacks() {
        let state = eval_source(
            r#"
            var.set("result", "init")
            try {
                assert_eq(1, 2)
            } fallback {
                assert_eq(3, 4)
            } fallback {
                var.set("result", "second fallback")
            }
            "#,
        )
        .unwrap();
        assert_eq!(
            state.var_get("result").unwrap(),
            Value::String("second fallback".to_string())
        );
    }

    #[test]
    fn test_eval_nested_try_blocks() {
        let state = eval_source(
            r#"
            var.set("result", "init")
            try {
                try {
                    assert_eq(1, 2)
                } fallback {
                    var.set("result", "inner fallback ran")
                }
            } fallback {
                var.set("result", "outer fallback")
            }
            "#,
        )
        .unwrap();
        assert_eq!(
            state.var_get("result").unwrap(),
            Value::String("inner fallback ran".to_string())
        );
    }

    #[test]
    fn test_eval_loop_with_terminate() {
        let state = eval_source(
            r#"
            var.set("count", 0)
            loop {
                var.set("count", math.add(var.get("count"), 1))
                try {
                    assert_eq(var.get("count"), 3)
                    terminate
                } fallback {
                }
            }
            "#,
        )
        .unwrap();
        assert_eq!(state.var_get("count").unwrap(), Value::Number(3.0));
    }

    #[test]
    fn test_eval_terminate() {
        let result = eval_source(
            r#"
            var.set("before", true)
            terminate
            var.set("after", true)
            "#,
        );
        assert!(result.is_ok());
        let state = result.unwrap();
        assert_eq!(state.var_get("before").unwrap(), Value::Boolean(true));
        assert!(state.var_get("after").is_err());
    }

    // --- Assertion Tests ---

    #[test]
    fn test_eval_assert_eq_pass() {
        assert!(eval_source("assert_eq(1, 1)").is_ok());
    }

    #[test]
    fn test_eval_assert_eq_fail() {
        let err = eval_expect_err("assert_eq(1, 2)");
        assert!(format!("{}", err).contains("1 != 2"));
    }

    #[test]
    fn test_eval_assert_neq_pass() {
        assert!(eval_source("assert_neq(1, 2)").is_ok());
    }

    #[test]
    fn test_eval_assert_neq_fail() {
        let err = eval_expect_err("assert_neq(1, 1)");
        assert!(format!("{}", err).contains("=="));
    }

    #[test]
    fn test_eval_assert_gt_pass() {
        assert!(eval_source("assert_gt(2, 1)").is_ok());
    }

    #[test]
    fn test_eval_assert_gt_fail() {
        let err = eval_expect_err("assert_gt(1, 2)");
        assert!(format!("{}", err).contains("!>"));
    }

    #[test]
    fn test_eval_assert_lt_pass() {
        assert!(eval_source("assert_lt(1, 2)").is_ok());
    }

    #[test]
    fn test_eval_assert_lt_fail() {
        let err = eval_expect_err("assert_lt(2, 1)");
        assert!(format!("{}", err).contains("!<"));
    }

    #[test]
    fn test_eval_assert_match_pass() {
        assert!(eval_source(r#"assert_match("hello.*", "hello world")"#).is_ok());
    }

    #[test]
    fn test_eval_assert_match_fail() {
        let err = eval_expect_err(r#"assert_match("hello.*", "goodbye")"#);
        assert!(format!("{}", err).contains("does not match"));
    }

    // --- Error Cases ---

    #[test]
    fn test_eval_var_not_found() {
        let err = eval_expect_err("var.set(\"x\", var.get(\"nonexistent\"))");
        assert!(format!("{}", err).contains("nonexistent"));
    }

    #[test]
    fn test_eval_static_not_found() {
        let err = eval_expect_err("var.set(\"x\", static.get(\"nonexistent\"))");
        assert!(format!("{}", err).contains("nonexistent"));
    }

    #[test]
    fn test_eval_unknown_function() {
        let err = eval_expect_err("nonexistent_func()");
        assert!(format!("{}", err).contains("nonexistent_func"));
    }

    #[test]
    fn test_eval_index_out_of_bounds() {
        let err = eval_expect_err(r#"list.get([], 0)"#);
        assert!(format!("{}", err).contains("out of bounds"));
    }

    #[test]
    fn test_eval_division_by_zero_mod() {
        assert!(eval_source(r#"math.mod(1, 0)"#).is_err());
    }

    // --- Complex Programs ---

    #[test]
    fn test_eval_comprehensive_program() {
        let state = eval_source(
            r#"
            var.set("counter", 0)
            var.set("results", [])
            loop {
                var.set("counter", math.add(var.get("counter"), 1))
                var.set("results", list.push(var.get("results"), var.get("counter")))
                try {
                    assert_eq(var.get("counter"), 5)
                    terminate
                } fallback {
                }
            }
            "#,
        )
        .unwrap();
        assert_eq!(state.var_get("counter").unwrap(), Value::Number(5.0));
        match state.var_get("results").unwrap() {
            Value::List(items) => assert_eq!(items.len(), 5),
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_eval_var_overwrite() {
        let state = eval_source(
            r#"
            var.set("x", 1)
            var.set("x", 2)
            "#,
        )
        .unwrap();
        assert_eq!(state.var_get("x").unwrap(), Value::Number(2.0));
    }

    #[test]
    fn test_eval_static_var_independent() {
        let state = eval_source(
            r#"
            var.set("x", 1)
            static.set("x", 2)
            "#,
        )
        .unwrap();
        assert_eq!(state.var_get("x").unwrap(), Value::Number(1.0));
        assert_eq!(state.static_get("x").unwrap(), Value::Number(2.0));
    }

    #[test]
    fn test_eval_nested_function_calls() {
        let state = eval_source(
            r#"
            var.set("result", math.add(math.mul(2, 3), math.div(10, 2)))
            "#,
        )
        .unwrap();
        assert_eq!(state.var_get("result").unwrap(), Value::Number(11.0));
    }

    #[test]
    fn test_eval_string_methods_in_expr() {
        let state = eval_source(
            r#"
            var.set("msg", string.concat(string.to_upper("hello"), " WORLD"))
            "#,
        )
        .unwrap();
        assert_eq!(
            state.var_get("msg").unwrap(),
            Value::String("HELLO WORLD".to_string())
        );
    }
}
