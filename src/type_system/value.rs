use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::type_system::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    #[default]
    Null,
}

impl Value {
    pub fn r#type(&self) -> Type {
        match self {
            Self::String(_) => Type::String,
            Self::Number(_) => Type::Number,
            Self::Boolean(_) => Type::Boolean,
            Self::List(_) => Type::List,
            Self::Map(_) => Type::Map,
            Self::Null => Type::Null,
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&Vec<Value>> {
        match self {
            Self::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Self::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Boolean(b) => *b,
            Self::Null => false,
            Self::String(s) => !s.is_empty(),
            Self::Number(n) => *n != 0.0,
            Self::List(l) => !l.is_empty(),
            Self::Map(m) => !m.is_empty(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => write!(f, "{}", s),
            Self::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Self::Boolean(b) => write!(f, "{}", b),
            Self::List(l) => {
                let items: Vec<String> = l.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Self::Map(m) => {
                let items: Vec<String> =
                    m.iter().map(|(k, v)| format!("\"{}\": {}", k, v)).collect();
                write!(f, "{{{}}}", items.join(", "))
            }
            Self::Null => write!(f, "null"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type() {
        assert_eq!(Value::String("test".to_string()).r#type(), Type::String);
        assert_eq!(Value::Number(42.0).r#type(), Type::Number);
        assert_eq!(Value::Boolean(true).r#type(), Type::Boolean);
        assert_eq!(Value::List(vec![]).r#type(), Type::List);
        assert_eq!(Value::Map(HashMap::new()).r#type(), Type::Map);
        assert_eq!(Value::Null.r#type(), Type::Null);
    }

    #[test]
    fn test_is_truthy() {
        assert!(!Value::Null.is_truthy());
        assert!(!Value::Boolean(false).is_truthy());
        assert!(Value::Boolean(true).is_truthy());
        assert!(Value::String("hello".to_string()).is_truthy());
        assert!(!Value::String(String::new()).is_truthy());
        assert!(Value::Number(1.0).is_truthy());
        assert!(!Value::Number(0.0).is_truthy());
        assert!(Value::Number(-1.0).is_truthy());
        assert!(Value::List(vec![Value::Number(1.0)]).is_truthy());
        assert!(!Value::List(vec![]).is_truthy());
        let mut map = HashMap::new();
        map.insert("a".to_string(), Value::Number(1.0));
        assert!(Value::Map(map).is_truthy());
        assert!(!Value::Map(HashMap::new()).is_truthy());
    }

    #[test]
    fn test_display() {
        assert_eq!(Value::Number(42.0).to_string(), "42");
        assert_eq!(Value::Number(42.5).to_string(), "42.5");
        assert_eq!(Value::Boolean(true).to_string(), "true");
        assert_eq!(Value::Boolean(false).to_string(), "false");
        assert_eq!(Value::Null.to_string(), "null");
        assert_eq!(Value::String("hello".to_string()).to_string(), "hello");
    }

    #[test]
    fn test_display_list() {
        let list = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);
        let display = list.to_string();
        assert_eq!(display, "[1, 2]");
    }

    #[test]
    fn test_display_empty_list() {
        let list = Value::List(vec![]);
        assert_eq!(list.to_string(), "[]");
    }

    #[test]
    fn test_display_map() {
        let mut map = HashMap::new();
        map.insert("a".to_string(), Value::Number(1.0));
        let display = Value::Map(map).to_string();
        assert!(display.contains("\"a\""));
        assert!(display.contains("1"));
    }

    #[test]
    fn test_display_nested() {
        let inner = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);
        let outer = Value::List(vec![inner, Value::String("hello".to_string())]);
        let display = outer.to_string();
        assert!(display.contains("[1, 2]"));
        assert!(display.contains("hello"));
    }

    #[test]
    fn test_as_string() {
        assert_eq!(
            Value::String("hello".to_string()).as_string(),
            Some(&"hello".to_string())
        );
        assert_eq!(Value::Number(42.0).as_string(), None);
    }

    #[test]
    fn test_as_number() {
        assert_eq!(Value::Number(42.0).as_number(), Some(42.0));
        assert_eq!(Value::String("42".to_string()).as_number(), None);
    }

    #[test]
    fn test_as_bool() {
        assert_eq!(Value::Boolean(true).as_bool(), Some(true));
        assert_eq!(Value::Boolean(false).as_bool(), Some(false));
        assert_eq!(Value::Null.as_bool(), None);
    }

    #[test]
    fn test_as_list() {
        let items = vec![Value::Number(1.0)];
        assert!(Value::List(items.clone()).as_list().is_some());
        assert!(Value::Null.as_list().is_none());
    }

    #[test]
    fn test_as_map() {
        let map = HashMap::new();
        assert!(Value::Map(map).as_map().is_some());
        assert!(Value::Null.as_map().is_none());
    }

    #[test]
    fn test_default_value() {
        let val: Value = Default::default();
        assert_eq!(val, Value::Null);
    }

    #[test]
    fn test_clone_equality() {
        let original = Value::List(vec![Value::String("a".to_string()), Value::Number(1.0)]);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_number_display_integer_vs_float() {
        assert_eq!(Value::Number(0.0).to_string(), "0");
        assert_eq!(Value::Number(1.0).to_string(), "1");
        assert_eq!(Value::Number(-1.0).to_string(), "-1");
        assert_eq!(Value::Number(0.5).to_string(), "0.5");
        assert_eq!(Value::Number(100.25).to_string(), "100.25");
    }
}
