use crate::ast::{AssertKind, Expr, FallbackBlock, Program, Stmt};
use crate::lexer::token::{Token, TokenType};
use crate::{CorvoError, CorvoResult};
use std::collections::HashMap;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    in_prep_block: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            in_prep_block: false,
        }
    }

    pub fn parse(&mut self) -> CorvoResult<Program> {
        let mut statements = Vec::new();
        let mut seen_non_prep = false;

        while !self.is_at_end() {
            self.skip_comments();
            if self.is_at_end() {
                break;
            }

            if matches!(self.peek().token_type, TokenType::Prep) {
                if seen_non_prep {
                    return Err(self.error("prep block must come before all other statements"));
                }
                let stmt = self.parse_prep()?;
                statements.push(stmt);
            } else {
                seen_non_prep = true;
                if let Some(stmt) = self.parse_statement()? {
                    statements.push(stmt);
                }
            }
        }

        Ok(Program::new(statements))
    }

    fn parse_statement(&mut self) -> CorvoResult<Option<Stmt>> {
        self.skip_comments();

        if self.is_at_end() {
            return Ok(None);
        }

        let stmt = match &self.peek().token_type {
            TokenType::Prep => {
                return Err(self.error("prep block can only appear at the top level of a program"));
            }
            TokenType::Static => self.parse_static_set()?,
            TokenType::Var => self.parse_var_set()?,
            TokenType::Try => self.parse_try_block()?,
            TokenType::Loop => self.parse_loop()?,
            TokenType::Browse => self.parse_browse()?,
            TokenType::Terminate => self.parse_terminate()?,
            TokenType::DontPanic => self.parse_dont_panic()?,
            TokenType::AssertEq
            | TokenType::AssertNeq
            | TokenType::AssertGt
            | TokenType::AssertLt
            | TokenType::AssertMatch => self.parse_assert()?,
            TokenType::At => {
                // @name = value       → VarSet shortcut
                // @name[index] = val  → VarIndexSet shortcut
                // @name               → ExprStmt (VarGet shortcut)
                let is_simple_assignment = matches!(
                    self.tokens.get(self.current + 1).map(|t| &t.token_type),
                    Some(TokenType::Identifier(_))
                ) && matches!(
                    self.tokens.get(self.current + 2).map(|t| &t.token_type),
                    Some(TokenType::Equals)
                );
                let is_index_assignment = matches!(
                    self.tokens.get(self.current + 1).map(|t| &t.token_type),
                    Some(TokenType::Identifier(_))
                ) && matches!(
                    self.tokens.get(self.current + 2).map(|t| &t.token_type),
                    Some(TokenType::LeftBracket)
                );
                if is_simple_assignment {
                    self.parse_at_var_set()?
                } else if is_index_assignment {
                    self.parse_at_index_set_or_expr()?
                } else {
                    self.parse_expr_statement()?
                }
            }
            _ => self.parse_expr_statement()?,
        };

        Ok(Some(stmt))
    }

    // --- Statement Parsers ---

    fn parse_prep(&mut self) -> CorvoResult<Stmt> {
        self.advance(); // consume 'prep'
        self.consume(TokenType::LeftBrace, "Expected '{' after 'prep'")?;
        self.in_prep_block = true;
        let body = self.parse_block_body("prep block");
        self.in_prep_block = false;
        Ok(Stmt::PrepBlock { body: body? })
    }

    fn parse_static_set(&mut self) -> CorvoResult<Stmt> {
        if !self.in_prep_block {
            return Err(self.error("static.set() can only be used inside a prep block"));
        }
        self.advance(); // consume 'static'

        self.consume(TokenType::Dot, "Expected '.' after 'static'")?;
        self.consume(
            TokenType::Identifier("set".to_string()),
            "Expected 'set' after '.'",
        )?;

        self.consume(TokenType::LeftParen, "Expected '(' after 'set'")?;

        let name = self.parse_string_arg("static.set name")?;
        self.consume(TokenType::Comma, "Expected ',' after name")?;

        let value = self.parse_expression()?;

        self.consume(TokenType::RightParen, "Expected ')' after value")?;

        Ok(Stmt::StaticSet { name, value })
    }

    fn parse_var_set(&mut self) -> CorvoResult<Stmt> {
        self.advance(); // consume 'var'

        self.consume(TokenType::Dot, "Expected '.' after 'var'")?;
        self.consume(
            TokenType::Identifier("set".to_string()),
            "Expected 'set' after '.'",
        )?;

        self.consume(TokenType::LeftParen, "Expected '(' after 'set'")?;

        let name = self.parse_string_arg("var.set name")?;
        self.consume(TokenType::Comma, "Expected ',' after name")?;

        let value = self.parse_expression()?;

        self.consume(TokenType::RightParen, "Expected ')' after value")?;

        Ok(Stmt::VarSet { name, value })
    }

    fn parse_try_block(&mut self) -> CorvoResult<Stmt> {
        self.advance(); // consume 'try'
        self.consume(TokenType::LeftBrace, "Expected '{' after 'try'")?;

        let body = self.parse_block_body("try block")?;

        let mut fallbacks = Vec::new();

        while self.match_token(TokenType::Fallback) {
            self.consume(TokenType::LeftBrace, "Expected '{' after 'fallback'")?;
            let fb_body = self.parse_block_body("fallback block")?;
            fallbacks.push(FallbackBlock { body: fb_body });
        }

        Ok(Stmt::TryBlock { body, fallbacks })
    }

    fn parse_loop(&mut self) -> CorvoResult<Stmt> {
        self.advance(); // consume 'loop'
        self.consume(TokenType::LeftBrace, "Expected '{' after 'loop'")?;

        let body = self.parse_block_body("loop block")?;

        Ok(Stmt::Loop { body })
    }

    fn parse_browse(&mut self) -> CorvoResult<Stmt> {
        self.advance(); // consume 'browse'
        self.consume(TokenType::LeftParen, "Expected '(' after 'browse'")?;

        let iterable = self.parse_expression()?;

        self.consume(TokenType::Comma, "Expected ',' after iterable")?;

        let key = match &self.peek().token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => return Err(self.error("Expected identifier for key variable name")),
        };
        self.advance(); // consume key identifier

        self.consume(TokenType::Comma, "Expected ',' after key name")?;

        let value = match &self.peek().token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => return Err(self.error("Expected identifier for value variable name")),
        };
        self.advance(); // consume value identifier

        self.consume(TokenType::RightParen, "Expected ')' after value name")?;
        self.consume(TokenType::LeftBrace, "Expected '{' after ')'")?;

        let body = self.parse_block_body("browse block")?;

        Ok(Stmt::Browse {
            iterable,
            key,
            value,
            body,
        })
    }

    fn parse_terminate(&mut self) -> CorvoResult<Stmt> {
        self.advance(); // consume 'terminate'
        Ok(Stmt::Terminate)
    }

    fn parse_dont_panic(&mut self) -> CorvoResult<Stmt> {
        self.advance(); // consume 'dont_panic'
        self.consume(TokenType::LeftBrace, "Expected '{' after 'dont_panic'")?;
        let body = self.parse_block_body("dont_panic block")?;
        Ok(Stmt::DontPanic { body })
    }

    fn parse_at_var_set(&mut self) -> CorvoResult<Stmt> {
        self.advance(); // consume '@'
        let name = match &self.peek().token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => return Err(self.error("Expected variable name after '@'")),
        };
        self.advance(); // consume identifier
        self.consume(TokenType::Equals, "Expected '=' after variable name")?;
        let value = self.parse_expression()?;
        Ok(Stmt::VarSet { name, value })
    }

    fn parse_at_index_set_or_expr(&mut self) -> CorvoResult<Stmt> {
        self.advance(); // consume '@'
        let name = match &self.peek().token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => return Err(self.error("Expected variable name after '@'")),
        };
        self.advance(); // consume identifier
        self.consume(TokenType::LeftBracket, "Expected '[' after variable name")?;
        let index = self.parse_expression()?;
        self.consume(TokenType::RightBracket, "Expected ']' after index")?;

        if self.match_token(TokenType::Equals) {
            // @name[index] = value  →  VarIndexSet
            let value = self.parse_expression()?;
            Ok(Stmt::VarIndexSet { name, index, value })
        } else {
            // @name[index] used as an expression statement (read-only)
            let access = Expr::IndexAccess {
                target: Box::new(Expr::VarGet { name }),
                index: Box::new(index),
            };
            Ok(Stmt::ExprStmt { expr: access })
        }
    }

    fn parse_assert(&mut self) -> CorvoResult<Stmt> {
        let kind = match self.peek().token_type {
            TokenType::AssertEq => AssertKind::Eq,
            TokenType::AssertNeq => AssertKind::Neq,
            TokenType::AssertGt => AssertKind::Gt,
            TokenType::AssertLt => AssertKind::Lt,
            TokenType::AssertMatch => AssertKind::Match,
            _ => return Err(self.error("Expected assertion")),
        };
        self.advance();

        self.consume(TokenType::LeftParen, "Expected '(' after assertion")?;

        let mut args = Vec::new();
        if !self.check(TokenType::RightParen) {
            args.push(self.parse_expression()?);
            while self.match_token(TokenType::Comma) {
                args.push(self.parse_expression()?);
            }
        }

        self.consume(TokenType::RightParen, "Expected ')' after arguments")?;

        Ok(Stmt::Assert { kind, args })
    }

    fn parse_expr_statement(&mut self) -> CorvoResult<Stmt> {
        let expr = self.parse_expression()?;
        Ok(Stmt::ExprStmt { expr })
    }

    fn parse_block_body(&mut self, context: &str) -> CorvoResult<Vec<Stmt>> {
        let mut body = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            if let Some(stmt) = self.parse_statement()? {
                body.push(stmt);
            }
        }

        self.consume(
            TokenType::RightBrace,
            &format!("Expected '}}' after {}", context),
        )?;
        Ok(body)
    }

    // --- Expression Parsers ---

    fn parse_expression(&mut self) -> CorvoResult<Expr> {
        let expr = self.parse_primary()?;
        self.parse_postfix(expr)
    }

    fn parse_postfix(&mut self, expr: Expr) -> CorvoResult<Expr> {
        if self.match_token(TokenType::LeftBracket) {
            let index = self.parse_expression()?;
            self.consume(TokenType::RightBracket, "Expected ']' after index")?;
            let indexed = Expr::IndexAccess {
                target: Box::new(expr),
                index: Box::new(index),
            };
            return self.parse_postfix(indexed);
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> CorvoResult<Expr> {
        let token = self.peek().clone();

        match &token.token_type {
            TokenType::String(s) => {
                self.advance();
                Ok(Expr::Literal {
                    value: crate::type_system::Value::String(s.clone()),
                })
            }
            TokenType::StringInterpolation(parts) => {
                let parts = parts.clone();
                self.advance();
                self.parse_string_interpolation(parts)
            }
            TokenType::Number(n) => {
                self.advance();
                Ok(Expr::Literal {
                    value: crate::type_system::Value::Number(*n),
                })
            }
            TokenType::Boolean(b) => {
                self.advance();
                Ok(Expr::Literal {
                    value: crate::type_system::Value::Boolean(*b),
                })
            }
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance();
                if self.match_token(TokenType::Dot) {
                    self.parse_method_call(name)
                } else if self.match_token(TokenType::LeftParen) {
                    self.parse_function_call(name)
                } else {
                    Ok(Expr::VarGet { name })
                }
            }
            TokenType::Var => {
                self.advance();
                self.parse_var_get()
            }
            TokenType::Static => {
                self.advance();
                self.parse_static_get()
            }
            TokenType::LeftBracket => self.parse_list_literal(),
            TokenType::LeftBrace => self.parse_map_literal(),
            TokenType::At => {
                self.advance(); // consume '@'
                match &self.peek().token_type {
                    TokenType::Identifier(s) => {
                        let name = s.clone();
                        self.advance(); // consume identifier
                        Ok(Expr::VarGet { name })
                    }
                    _ => Err(self.error("Expected variable name after '@'")),
                }
            }
            _ => Err(self.error(format!("Unexpected token: {}", token.token_type))),
        }
    }

    fn parse_string_interpolation(&self, parts: Vec<Token>) -> CorvoResult<Expr> {
        let mut expr_parts = Vec::new();

        for part in &parts {
            match &part.token_type {
                TokenType::String(s) => {
                    if !s.is_empty() {
                        expr_parts.push(Expr::Literal {
                            value: crate::type_system::Value::String(s.clone()),
                        });
                    }
                }
                TokenType::StringInterpolation(expr_tokens) => {
                    let mut sub_parser = Parser::new(expr_tokens.clone());
                    let expr = sub_parser.parse_expression()?;
                    expr_parts.push(expr);
                }
                _ => {}
            }
        }

        if expr_parts.len() == 1 {
            if let Expr::Literal { .. } = &expr_parts[0] {
                return Ok(expr_parts.into_iter().next().unwrap());
            }
        }

        Ok(Expr::StringInterpolation { parts: expr_parts })
    }

    fn parse_var_get(&mut self) -> CorvoResult<Expr> {
        self.consume(TokenType::Dot, "Expected '.' after 'var'")?;
        self.consume(
            TokenType::Identifier("get".to_string()),
            "Expected 'get' after 'var.'",
        )?;
        self.consume(TokenType::LeftParen, "Expected '(' after 'get'")?;

        let name = self.parse_string_arg("var.get")?;

        self.consume(TokenType::RightParen, "Expected ')' after name")?;

        Ok(Expr::VarGet { name })
    }

    fn parse_static_get(&mut self) -> CorvoResult<Expr> {
        self.consume(TokenType::Dot, "Expected '.' after 'static'")?;
        self.consume(
            TokenType::Identifier("get".to_string()),
            "Expected 'get' after 'static.'",
        )?;
        self.consume(TokenType::LeftParen, "Expected '(' after 'get'")?;

        let name = self.parse_string_arg("static.get")?;

        self.consume(TokenType::RightParen, "Expected ')' after name")?;

        Ok(Expr::StaticGet { name })
    }

    fn parse_list_literal(&mut self) -> CorvoResult<Expr> {
        self.advance(); // consume '['

        let mut items = Vec::new();

        if !self.check(TokenType::RightBracket) {
            items.push(self.parse_expression()?);
            while self.match_token(TokenType::Comma) {
                items.push(self.parse_expression()?);
            }
        }

        self.consume(TokenType::RightBracket, "Expected ']' after list literal")?;

        Ok(Expr::FunctionCall {
            name: "__list__".to_string(),
            args: items,
            named_args: HashMap::new(),
        })
    }

    fn parse_map_literal(&mut self) -> CorvoResult<Expr> {
        self.advance(); // consume '{'

        let mut items = Vec::new();

        if !self.check(TokenType::RightBrace) {
            loop {
                let key = self.parse_expression()?;
                self.consume(TokenType::Colon, "Expected ':' after map key")?;
                let value = self.parse_expression()?;
                items.push(key);
                items.push(value);

                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightBrace, "Expected '}' after map literal")?;

        Ok(Expr::FunctionCall {
            name: "__map__".to_string(),
            args: items,
            named_args: HashMap::new(),
        })
    }

    // --- Call Parsers ---

    fn parse_function_call(&mut self, name: String) -> CorvoResult<Expr> {
        let (args, named_args) = self.parse_call_args()?;
        self.consume(TokenType::RightParen, "Expected ')' after arguments")?;

        Ok(Expr::FunctionCall {
            name,
            args,
            named_args,
        })
    }

    fn parse_method_call(&mut self, obj: String) -> CorvoResult<Expr> {
        let method = match self.peek().token_type.clone() {
            TokenType::Identifier(s) => s,
            _ => return Err(self.error("Expected method name")),
        };
        self.advance();

        self.consume(TokenType::LeftParen, "Expected '(' after method")?;

        let (args, named_args) = self.parse_call_args()?;
        self.consume(TokenType::RightParen, "Expected ')' after arguments")?;

        let func_name = format!("{}.{}", obj, method);
        Ok(Expr::FunctionCall {
            name: func_name,
            args,
            named_args,
        })
    }

    fn parse_call_args(&mut self) -> CorvoResult<(Vec<Expr>, HashMap<String, Expr>)> {
        let mut args = Vec::new();
        let mut named_args = HashMap::new();

        if self.check(TokenType::RightParen) {
            return Ok((args, named_args));
        }

        loop {
            if self.is_named_param() {
                let param_name = match self.peek().token_type.clone() {
                    TokenType::Identifier(s) => s,
                    _ => break,
                };
                self.advance(); // consume identifier
                self.consume(TokenType::Colon, "Expected ':' after named parameter")?;
                let value = self.parse_expression()?;
                named_args.insert(param_name, value);
            } else {
                args.push(self.parse_expression()?);
            }

            if !self.match_token(TokenType::Comma) {
                break;
            }
        }

        Ok((args, named_args))
    }

    // --- Helpers ---

    fn parse_string_arg(&mut self, context: &str) -> CorvoResult<String> {
        match &self.peek().token_type {
            TokenType::String(s) => {
                let name = s.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(self.error(format!("Expected string for {}", context))),
        }
    }

    fn is_named_param(&self) -> bool {
        matches!(self.peek().token_type, TokenType::Identifier(_))
            && self.peek_next().token_type == TokenType::Colon
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_next(&self) -> &Token {
        if self.current + 1 < self.tokens.len() {
            &self.tokens[self.current + 1]
        } else {
            &self.tokens[self.current]
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::Eof)
    }

    fn check(&self, token_type: TokenType) -> bool {
        std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(&token_type)
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> CorvoResult<()> {
        if self.check(token_type) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(format!("{} (got {})", message, self.peek().token_type)))
        }
    }

    fn error(&self, message: impl Into<String>) -> CorvoError {
        let span = self.peek().span;
        CorvoError::parsing(message.into()).with_span(span)
    }

    fn skip_comments(&mut self) {
        while matches!(self.peek().token_type, TokenType::Comment(_)) {
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_source(source: &str) -> CorvoResult<Program> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    fn parse_expect_err(source: &str) -> CorvoError {
        parse_source(source).expect_err(&format!("Expected parse error for: {}", source))
    }

    // --- Basic Statement Tests ---

    #[test]
    fn test_parse_var_set() {
        let program = parse_source(r#"var.set("x", "hello")"#).unwrap();
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Stmt::VarSet { name, .. } => assert_eq!(name, "x"),
            _ => panic!("Expected VarSet"),
        }
    }

    #[test]
    fn test_parse_static_set() {
        let program = parse_source(r#"prep { static.set("key", 42) }"#).unwrap();
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Stmt::PrepBlock { body } => {
                assert_eq!(body.len(), 1);
                match &body[0] {
                    Stmt::StaticSet { name, .. } => assert_eq!(name, "key"),
                    _ => panic!("Expected StaticSet inside PrepBlock"),
                }
            }
            _ => panic!("Expected PrepBlock"),
        }
    }

    #[test]
    fn test_parse_prep_block() {
        let program = parse_source(
            r#"
            prep {
                static.set("config", "value")
            }
            var.set("x", 1)
            "#,
        )
        .unwrap();
        assert_eq!(program.statements.len(), 2);
        match &program.statements[0] {
            Stmt::PrepBlock { body } => assert_eq!(body.len(), 1),
            _ => panic!("Expected PrepBlock"),
        }
    }

    #[test]
    fn test_parse_prep_must_come_first() {
        let err = parse_expect_err(
            r#"
            var.set("x", 1)
            prep {
                static.set("config", "value")
            }
            "#,
        );
        let msg = format!("{}", err);
        assert!(msg.contains("prep block must come before all other statements"));
    }

    #[test]
    fn test_parse_prep_not_allowed_in_nested_block() {
        let err = parse_expect_err(
            r#"
            try {
                prep { static.set("x", 1) }
            } fallback {}
            "#,
        );
        let msg = format!("{}", err);
        assert!(msg.contains("prep block can only appear at the top level"));
    }

    #[test]
    fn test_parse_static_set_outside_prep_error() {
        let err = parse_expect_err(r#"static.set("key", 42)"#);
        let msg = format!("{}", err);
        assert!(msg.contains("static.set() can only be used inside a prep block"));
    }

    #[test]
    fn test_parse_terminate() {
        let program = parse_source("terminate").unwrap();
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Stmt::Terminate => {}
            _ => panic!("Expected Terminate"),
        }
    }

    #[test]
    fn test_parse_multiple_statements() {
        let program = parse_source(
            r#"
            var.set("x", 1)
            var.set("y", 2)
            sys.echo(var.get("x"))
            "#,
        )
        .unwrap();
        assert_eq!(program.statements.len(), 3);
    }

    // --- Expression Tests ---

    #[test]
    fn test_parse_var_get_expr() {
        let program = parse_source(r#"sys.echo(var.get("x"))"#).unwrap();
        match &program.statements[0] {
            Stmt::ExprStmt { expr } => match expr {
                Expr::FunctionCall { name, args, .. } => {
                    assert_eq!(name, "sys.echo");
                    assert_eq!(args.len(), 1);
                    match &args[0] {
                        Expr::VarGet { name } => assert_eq!(name, "x"),
                        _ => panic!("Expected VarGet inside sys.echo"),
                    }
                }
                _ => panic!("Expected FunctionCall"),
            },
            _ => panic!("Expected ExprStmt"),
        }
    }

    #[test]
    fn test_parse_method_call() {
        let program = parse_source(r#"string.concat("hello", " world")"#).unwrap();
        match &program.statements[0] {
            Stmt::ExprStmt { expr } => match expr {
                Expr::FunctionCall { name, args, .. } => {
                    assert_eq!(name, "string.concat");
                    assert_eq!(args.len(), 2);
                }
                _ => panic!("Expected FunctionCall"),
            },
            _ => panic!("Expected ExprStmt"),
        }
    }

    #[test]
    fn test_parse_function_with_named_args() {
        let program = parse_source(r#"http.get(url: "https://example.com")"#).unwrap();
        match &program.statements[0] {
            Stmt::ExprStmt { expr } => match expr {
                Expr::FunctionCall {
                    name,
                    args,
                    named_args,
                } => {
                    assert_eq!(name, "http.get");
                    assert!(args.is_empty());
                    assert!(named_args.contains_key("url"));
                }
                _ => panic!("Expected FunctionCall"),
            },
            _ => panic!("Expected ExprStmt"),
        }
    }

    #[test]
    fn test_parse_mixed_positional_and_named_args() {
        let program = parse_source(r#"my_func(1, 2, key: "value")"#).unwrap();
        match &program.statements[0] {
            Stmt::ExprStmt { expr } => match expr {
                Expr::FunctionCall {
                    args, named_args, ..
                } => {
                    assert_eq!(args.len(), 2);
                    assert_eq!(named_args.len(), 1);
                    assert!(named_args.contains_key("key"));
                }
                _ => panic!("Expected FunctionCall"),
            },
            _ => panic!("Expected ExprStmt"),
        }
    }

    // --- Block Tests ---

    #[test]
    fn test_parse_try_fallback() {
        let program = parse_source(
            r#"
            try {
                assert_eq(1, 1)
            } fallback {
                sys.echo("failed")
            }
            "#,
        )
        .unwrap();
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Stmt::TryBlock { body, fallbacks } => {
                assert_eq!(body.len(), 1);
                assert_eq!(fallbacks.len(), 1);
                assert_eq!(fallbacks[0].body.len(), 1);
            }
            _ => panic!("Expected TryBlock"),
        }
    }

    #[test]
    fn test_parse_try_multiple_fallbacks() {
        let program = parse_source(
            r#"
            try {
                assert_eq(1, 1)
            } fallback {
                assert_eq(2, 2)
            } fallback {
                sys.echo("all failed")
            }
            "#,
        )
        .unwrap();
        match &program.statements[0] {
            Stmt::TryBlock { fallbacks, .. } => {
                assert_eq!(fallbacks.len(), 2);
            }
            _ => panic!("Expected TryBlock"),
        }
    }

    #[test]
    fn test_parse_nested_try_blocks() {
        let program = parse_source(
            r#"
            try {
                try {
                    assert_eq(1, 1)
                } fallback {
                    sys.echo("inner")
                }
            } fallback {
                sys.echo("outer")
            }
            "#,
        )
        .unwrap();
        match &program.statements[0] {
            Stmt::TryBlock { body, fallbacks } => {
                assert_eq!(body.len(), 1);
                assert_eq!(fallbacks.len(), 1);
                match &body[0] {
                    Stmt::TryBlock { .. } => {}
                    _ => panic!("Expected nested TryBlock"),
                }
            }
            _ => panic!("Expected TryBlock"),
        }
    }

    #[test]
    fn test_parse_loop() {
        let program = parse_source(
            r#"
            loop {
                sys.echo("hello")
            }
            "#,
        )
        .unwrap();
        match &program.statements[0] {
            Stmt::Loop { body } => {
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected Loop"),
        }
    }

    #[test]
    fn test_parse_loop_with_multiple_stmts() {
        let program = parse_source(
            r#"
            loop {
                var.set("x", math.add(var.get("x"), 1))
                assert_eq(var.get("x"), 10)
                terminate
            }
            "#,
        )
        .unwrap();
        match &program.statements[0] {
            Stmt::Loop { body } => {
                assert_eq!(body.len(), 3);
            }
            _ => panic!("Expected Loop"),
        }
    }

    #[test]
    fn test_parse_browse_list() {
        let program = parse_source(
            r#"
            browse(var.get("items"), key, val) {
                sys.echo("hello")
            }
            "#,
        )
        .unwrap();
        match &program.statements[0] {
            Stmt::Browse {
                key, value, body, ..
            } => {
                assert_eq!(key, "key");
                assert_eq!(value, "val");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected Browse"),
        }
    }

    #[test]
    fn test_parse_browse_with_at_var() {
        let program = parse_source(
            r#"
            browse(@my_list, k, v) {
                sys.echo(@v)
            }
            "#,
        )
        .unwrap();
        match &program.statements[0] {
            Stmt::Browse {
                key, value, body, ..
            } => {
                assert_eq!(key, "k");
                assert_eq!(value, "v");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected Browse"),
        }
    }

    // --- Assertion Tests ---

    #[test]
    fn test_parse_assert_eq() {
        let program = parse_source("assert_eq(1, 2)").unwrap();
        match &program.statements[0] {
            Stmt::Assert { kind, args } => {
                assert_eq!(*kind, AssertKind::Eq);
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected Assert"),
        }
    }

    #[test]
    fn test_parse_all_assertions() {
        let assertions = [
            ("assert_eq(1, 1)", AssertKind::Eq),
            ("assert_neq(1, 2)", AssertKind::Neq),
            ("assert_gt(2, 1)", AssertKind::Gt),
            ("assert_lt(1, 2)", AssertKind::Lt),
            ("assert_match(\".*\", \"test\")", AssertKind::Match),
        ];

        for (source, expected_kind) in assertions {
            let program = parse_source(source).unwrap();
            match &program.statements[0] {
                Stmt::Assert { kind, .. } => assert_eq!(*kind, expected_kind),
                _ => panic!("Expected Assert for: {}", source),
            }
        }
    }

    // --- Literal Tests ---

    #[test]
    fn test_parse_empty_list() {
        let program = parse_source(r#"var.set("items", [])"#).unwrap();
        match &program.statements[0] {
            Stmt::VarSet { value, .. } => match value {
                Expr::FunctionCall { name, args, .. } => {
                    assert_eq!(name, "__list__");
                    assert!(args.is_empty());
                }
                _ => panic!("Expected FunctionCall for list literal"),
            },
            _ => panic!("Expected VarSet"),
        }
    }

    #[test]
    fn test_parse_list_with_items() {
        let program = parse_source(r#"var.set("items", [1, 2, 3])"#).unwrap();
        match &program.statements[0] {
            Stmt::VarSet { value, .. } => match value {
                Expr::FunctionCall { name, args, .. } => {
                    assert_eq!(name, "__list__");
                    assert_eq!(args.len(), 3);
                }
                _ => panic!("Expected FunctionCall"),
            },
            _ => panic!("Expected VarSet"),
        }
    }

    #[test]
    fn test_parse_empty_map() {
        let program = parse_source(r#"var.set("m", {})"#).unwrap();
        match &program.statements[0] {
            Stmt::VarSet { value, .. } => match value {
                Expr::FunctionCall { name, args, .. } => {
                    assert_eq!(name, "__map__");
                    assert!(args.is_empty());
                }
                _ => panic!("Expected FunctionCall for map literal"),
            },
            _ => panic!("Expected VarSet"),
        }
    }

    #[test]
    fn test_parse_map_with_entries() {
        let program = parse_source(r#"var.set("m", {"a": 1, "b": 2})"#).unwrap();
        match &program.statements[0] {
            Stmt::VarSet { value, .. } => match value {
                Expr::FunctionCall { name, args, .. } => {
                    assert_eq!(name, "__map__");
                    assert_eq!(args.len(), 4); // key, value, key, value
                }
                _ => panic!("Expected FunctionCall"),
            },
            _ => panic!("Expected VarSet"),
        }
    }

    // --- Index Access Tests ---

    #[test]
    fn test_parse_index_access() {
        let program = parse_source(r#"list.get(var.get("items"), 0)"#).unwrap();
        match &program.statements[0] {
            Stmt::ExprStmt { expr } => match expr {
                Expr::FunctionCall { name, args, .. } => {
                    assert_eq!(name, "list.get");
                    assert_eq!(args.len(), 2);
                }
                _ => panic!("Expected FunctionCall"),
            },
            _ => panic!("Expected ExprStmt"),
        }
    }

    #[test]
    fn test_parse_index_access_expr() {
        let program = parse_source("my_list[0]").unwrap();
        match &program.statements[0] {
            Stmt::ExprStmt { expr } => match expr {
                Expr::IndexAccess { target, index } => {
                    match target.as_ref() {
                        Expr::VarGet { name } => assert_eq!(name, "my_list"),
                        _ => panic!("Expected VarGet as index target"),
                    }
                    match index.as_ref() {
                        Expr::Literal { value } => {
                            assert_eq!(*value, crate::type_system::Value::Number(0.0))
                        }
                        _ => panic!("Expected number literal as index"),
                    }
                }
                _ => panic!("Expected IndexAccess"),
            },
            _ => panic!("Expected ExprStmt"),
        }
    }

    #[test]
    fn test_parse_chained_index_access() {
        let program = parse_source("matrix[0][1]").unwrap();
        match &program.statements[0] {
            Stmt::ExprStmt { expr } => match expr {
                Expr::IndexAccess { target, .. } => match target.as_ref() {
                    Expr::IndexAccess { .. } => {}
                    _ => panic!("Expected chained IndexAccess"),
                },
                _ => panic!("Expected IndexAccess"),
            },
            _ => panic!("Expected ExprStmt"),
        }
    }

    // --- Comments ---

    #[test]
    fn test_parse_comments() {
        let program = parse_source(
            r#"
            # This is a comment
            var.set("x", 1)
            # Another comment
            "#,
        )
        .unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_comment_between_statements() {
        let program = parse_source(
            r#"
            var.set("x", 1)
            # middle comment
            var.set("y", 2)
            "#,
        )
        .unwrap();
        assert_eq!(program.statements.len(), 2);
    }

    // --- Error Tests ---

    #[test]
    fn test_parse_error_missing_paren() {
        let err = parse_expect_err(r#"var.set("x", 1"#);
        let msg = format!("{}", err);
        assert!(msg.contains("Expected ')'"));
    }

    #[test]
    fn test_parse_error_missing_brace() {
        let err = parse_expect_err("try { sys.echo(1)");
        let msg = format!("{}", err);
        assert!(msg.contains("Expected '}'"));
    }

    #[test]
    fn test_parse_error_unexpected_token() {
        let err = parse_expect_err("+");
        let msg = format!("{}", err);
        assert!(msg.contains("Unexpected token"));
    }

    #[test]
    fn test_parse_error_var_set_missing_dot() {
        let err = parse_expect_err(r#"var set("x", 1)"#);
        let msg = format!("{}", err);
        assert!(msg.contains("Expected '.'"));
    }

    #[test]
    fn test_parse_error_missing_comma() {
        let err = parse_expect_err(r#"var.set("x" 1)"#);
        let msg = format!("{}", err);
        assert!(msg.contains("Expected ','"));
    }

    #[test]
    fn test_parse_error_static_set_missing_string() {
        let err = parse_expect_err("prep { static.set(42, 1) }");
        let msg = format!("{}", err);
        assert!(msg.contains("Expected string"));
    }

    // --- Edge Cases ---

    #[test]
    fn test_parse_empty_input() {
        let program = parse_source("").unwrap();
        assert_eq!(program.statements.len(), 0);
    }

    #[test]
    fn test_parse_whitespace_only() {
        let program = parse_source("   \n\t  \n  ").unwrap();
        assert_eq!(program.statements.len(), 0);
    }

    #[test]
    fn test_parse_comments_only() {
        let program = parse_source("# just a comment\n# another").unwrap();
        assert_eq!(program.statements.len(), 0);
    }

    #[test]
    fn test_parse_complex_program() {
        let program = parse_source(
            r#"
            # Initialize
            prep {
                static.set("max", 10)
            }
            var.set("counter", 0)

            # Main loop
            loop {
                var.set("counter", math.add(var.get("counter"), 1))
                try {
                    assert_eq(var.get("counter"), var.get("max"))
                    terminate
                } fallback {
                    sys.echo("counting...")
                }
            }
            "#,
        )
        .unwrap();
        assert_eq!(program.statements.len(), 3);
    }
}
