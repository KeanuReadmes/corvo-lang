use crate::type_system::Value;
use crate::{CorvoError, CorvoResult};
use std::collections::HashMap;

pub fn add(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    if args.len() < 2 {
        return Err(CorvoError::invalid_argument("add requires 2 arguments"));
    }

    let a = args[0]
        .as_number()
        .ok_or_else(|| CorvoError::r#type("add requires numbers"))?;
    let b = args[1]
        .as_number()
        .ok_or_else(|| CorvoError::r#type("add requires numbers"))?;

    Ok(Value::Number(a + b))
}

pub fn sub(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    if args.len() < 2 {
        return Err(CorvoError::invalid_argument("sub requires 2 arguments"));
    }

    let a = args[0]
        .as_number()
        .ok_or_else(|| CorvoError::r#type("sub requires numbers"))?;
    let b = args[1]
        .as_number()
        .ok_or_else(|| CorvoError::r#type("sub requires numbers"))?;

    Ok(Value::Number(a - b))
}

pub fn mul(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    if args.len() < 2 {
        return Err(CorvoError::invalid_argument("mul requires 2 arguments"));
    }

    let a = args[0]
        .as_number()
        .ok_or_else(|| CorvoError::r#type("mul requires numbers"))?;
    let b = args[1]
        .as_number()
        .ok_or_else(|| CorvoError::r#type("mul requires numbers"))?;

    Ok(Value::Number(a * b))
}

pub fn div(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    if args.len() < 2 {
        return Err(CorvoError::invalid_argument("div requires 2 arguments"));
    }

    let a = args[0]
        .as_number()
        .ok_or_else(|| CorvoError::r#type("div requires numbers"))?;
    let b = args[1]
        .as_number()
        .ok_or_else(|| CorvoError::r#type("div requires numbers"))?;

    if b == 0.0 {
        return Err(CorvoError::division_by_zero());
    }

    Ok(Value::Number(a / b))
}

pub fn modulo(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    if args.len() < 2 {
        return Err(CorvoError::invalid_argument("mod requires 2 arguments"));
    }

    let a = args[0]
        .as_number()
        .ok_or_else(|| CorvoError::r#type("mod requires numbers"))?;
    let b = args[1]
        .as_number()
        .ok_or_else(|| CorvoError::r#type("mod requires numbers"))?;

    if b == 0.0 {
        return Err(CorvoError::division_by_zero());
    }

    Ok(Value::Number(a % b))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_args() -> HashMap<String, Value> {
        HashMap::new()
    }

    #[test]
    fn test_add() {
        let args = vec![Value::Number(2.0), Value::Number(3.0)];
        assert_eq!(add(&args, &empty_args()).unwrap(), Value::Number(5.0));
    }

    #[test]
    fn test_add_negative() {
        let args = vec![Value::Number(-1.0), Value::Number(3.0)];
        assert_eq!(add(&args, &empty_args()).unwrap(), Value::Number(2.0));
    }

    #[test]
    fn test_sub() {
        let args = vec![Value::Number(10.0), Value::Number(3.0)];
        assert_eq!(sub(&args, &empty_args()).unwrap(), Value::Number(7.0));
    }

    #[test]
    fn test_mul() {
        let args = vec![Value::Number(4.0), Value::Number(5.0)];
        assert_eq!(mul(&args, &empty_args()).unwrap(), Value::Number(20.0));
    }

    #[test]
    fn test_div() {
        let args = vec![Value::Number(10.0), Value::Number(2.0)];
        assert_eq!(div(&args, &empty_args()).unwrap(), Value::Number(5.0));
    }

    #[test]
    fn test_div_by_zero() {
        let args = vec![Value::Number(10.0), Value::Number(0.0)];
        assert!(div(&args, &empty_args()).is_err());
    }

    #[test]
    fn test_modulo() {
        let args = vec![Value::Number(10.0), Value::Number(3.0)];
        assert_eq!(modulo(&args, &empty_args()).unwrap(), Value::Number(1.0));
    }

    #[test]
    fn test_mod_by_zero() {
        let args = vec![Value::Number(10.0), Value::Number(0.0)];
        assert!(modulo(&args, &empty_args()).is_err());
    }

    #[test]
    fn test_add_wrong_type() {
        let args = vec![Value::String("a".to_string()), Value::Number(1.0)];
        assert!(add(&args, &empty_args()).is_err());
    }

    #[test]
    fn test_add_too_few_args() {
        let args = vec![Value::Number(1.0)];
        assert!(add(&args, &empty_args()).is_err());
    }

    #[test]
    fn test_add_zero() {
        let args = vec![Value::Number(5.0), Value::Number(0.0)];
        assert_eq!(add(&args, &empty_args()).unwrap(), Value::Number(5.0));
    }

    #[test]
    fn test_mul_by_zero() {
        let args = vec![Value::Number(5.0), Value::Number(0.0)];
        assert_eq!(mul(&args, &empty_args()).unwrap(), Value::Number(0.0));
    }

    #[test]
    fn test_div_float() {
        let args = vec![Value::Number(7.0), Value::Number(2.0)];
        assert_eq!(div(&args, &empty_args()).unwrap(), Value::Number(3.5));
    }
}
