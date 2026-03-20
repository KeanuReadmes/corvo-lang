use crate::compiler::Evaluator;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::runtime::RuntimeState;
use crate::CorvoError;
use std::io::{self, Write};

pub struct Repl {
    state: RuntimeState,
    evaluator: Evaluator,
    multiline_buffer: String,
    brace_depth: usize,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            state: RuntimeState::new(),
            evaluator: Evaluator::new(),
            multiline_buffer: String::new(),
            brace_depth: 0,
        }
    }

    pub fn run(&mut self) {
        println!("Corvo Language REPL v{}", env!("CARGO_PKG_VERSION"));
        println!("Type 'exit' to quit, 'help' for commands.\n");

        loop {
            let prompt = if self.multiline_buffer.is_empty() {
                "corvo> "
            } else {
                "   ..> "
            };

            print!("{}", prompt);
            io::stdout().flush().unwrap();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }

            let input = input.trim_end_matches(['\n', '\r']).to_string();

            if input.trim().is_empty() && self.multiline_buffer.is_empty() {
                continue;
            }

            // Handle special commands only at top level
            if self.multiline_buffer.is_empty() {
                let trimmed = input.trim();

                if trimmed == "exit" || trimmed == "quit" {
                    println!("Goodbye!");
                    break;
                }

                if trimmed == "help" {
                    print_help();
                    continue;
                }

                if trimmed == "vars" {
                    self.print_vars();
                    continue;
                }

                if trimmed == "reset" {
                    self.state = RuntimeState::new();
                    self.evaluator = Evaluator::new();
                    println!("State cleared.");
                    continue;
                }
            }

            // Track brace depth for multi-line input
            self.brace_depth += input.chars().filter(|&c| c == '{').count();
            self.brace_depth = self
                .brace_depth
                .saturating_sub(input.chars().filter(|&c| c == '}').count());

            if !self.multiline_buffer.is_empty() {
                self.multiline_buffer.push('\n');
            }
            self.multiline_buffer.push_str(&input);

            // Execute when braces are balanced
            if self.brace_depth == 0 {
                let source = std::mem::take(&mut self.multiline_buffer);
                self.execute(&source);
            }
        }
    }

    fn execute(&mut self, source: &str) {
        let source = source.trim();
        if source.is_empty() {
            return;
        }

        match self.parse_and_run(source) {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    fn parse_and_run(&mut self, source: &str) -> Result<(), CorvoError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;

        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;

        self.evaluator.run(&program, &mut self.state)?;
        Ok(())
    }

    fn print_vars(&self) {
        let var_keys = self.state.var_keys();
        let static_keys = self.state.static_keys();

        if var_keys.is_empty() && static_keys.is_empty() {
            println!("No variables defined.");
            return;
        }

        if !var_keys.is_empty() {
            println!("Variables:");
            for key in &var_keys {
                if let Ok(val) = self.state.var_get(key) {
                    println!("  var.get(\"{}\") = {}", key, val);
                }
            }
        }

        if !static_keys.is_empty() {
            println!("Static Variables:");
            for key in &static_keys {
                if let Ok(val) = self.state.static_get(key) {
                    println!("  static.get(\"{}\") = {}", key, val);
                }
            }
        }
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}

pub fn run() {
    let mut repl = Repl::new();
    repl.run();
}

fn print_help() {
    println!(
        r#"Corvo REPL Commands:
  exit, quit  Exit the REPL
  help        Show this help message
  vars        List all defined variables
  reset       Clear all state

Language Basics:
  var.set("name", value)     Set a variable
  var.get("name")            Get a variable
  static.set("name", value)  Set a persistent variable
  static.get("name")         Get a persistent variable

Control Flow:
  try {{ ... }} fallback {{ ... }}   Error handling
  loop {{ ... }}                     Loop until terminate
  terminate                         Exit current loop
  assert_eq(a, b)                   Assertion

Types & Literals:
  "string"          String literal
  42, 3.14          Number literal
  true, false       Boolean literal
  [1, 2, 3]         List literal
  {{"a": 1}}         Map literal

String Interpolation:
  "Hello ${{var.get(\"name\")}}"

Built-in Modules:
  sys.echo(msg)                 Print a message
  math.add(a, b)                Math operations
  string.concat(a, b)           String operations
  list.push(list, item)         List operations
  map.set(map, key, value)      Map operations
  fs.read(path)                 File operations
  json.parse(data)              Data parsing
  crypto.hash(algo, data)       Hashing
  http.get(url)                 HTTP requests

Examples:
  var.set("name", "World")
  sys.echo("Hello, ${{var.get(\"name\")}}")
  var.set("x", math.add(1, 2))
  var.set("items", [1, 2, 3])
  try {{
    assert_eq(var.get("x"), 3)
  }} fallback {{
    sys.echo("assertion failed")
  }}"#
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_new() {
        let repl = Repl::new();
        assert!(repl.state.is_empty());
        assert!(repl.multiline_buffer.is_empty());
        assert_eq!(repl.brace_depth, 0);
    }

    #[test]
    fn test_repl_execute_var_set() {
        let mut repl = Repl::new();
        repl.execute(r#"var.set("x", 42)"#);
        assert_eq!(
            repl.state.var_get("x").unwrap(),
            crate::type_system::Value::Number(42.0)
        );
    }

    #[test]
    fn test_repl_persistent_state() {
        let mut repl = Repl::new();
        repl.execute(r#"var.set("x", 1)"#);
        repl.execute(r#"var.set("y", 2)"#);
        repl.execute(r#"var.set("sum", math.add(var.get("x"), var.get("y")))"#);
        assert_eq!(
            repl.state.var_get("sum").unwrap(),
            crate::type_system::Value::Number(3.0)
        );
    }

    #[test]
    fn test_repl_error_handling() {
        let mut repl = Repl::new();
        // Should not panic, just print error
        repl.execute("nonexistent_func()");
        assert!(repl.state.is_empty());
    }

    #[test]
    fn test_repl_reset() {
        let mut repl = Repl::new();
        repl.execute(r#"var.set("x", 42)"#);
        repl.state = RuntimeState::new();
        repl.evaluator = Evaluator::new();
        assert!(repl.state.is_empty());
    }

    #[test]
    fn test_repl_multiline_brace_tracking() {
        let mut repl = Repl::new();

        // Simulate multi-line input
        repl.brace_depth += 1; // {
        repl.multiline_buffer.push_str("try {\n");
        repl.brace_depth += 1; // {
        repl.multiline_buffer.push_str("loop {\n");
        repl.brace_depth -= 1; // }
        repl.multiline_buffer.push_str("}\n");
        repl.brace_depth -= 1; // }
        repl.multiline_buffer.push_str("}\n");

        assert_eq!(repl.brace_depth, 0);
    }

    #[test]
    fn test_repl_static_vars() {
        let mut repl = Repl::new();
        repl.execute(r#"prep { static.set("PI", 2.5) }"#);
        assert_eq!(
            repl.state.static_get("PI").unwrap(),
            crate::type_system::Value::Number(2.5)
        );
    }

    #[test]
    fn test_repl_try_fallback() {
        let mut repl = Repl::new();
        repl.execute(
            r#"
            var.set("result", "init")
            try {
                assert_eq(1, 2)
            } fallback {
                var.set("result", "fell back")
            }
            "#,
        );
        assert_eq!(
            repl.state.var_get("result").unwrap(),
            crate::type_system::Value::String("fell back".to_string())
        );
    }
}
