use crate::type_system::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Scope {
    vars: HashMap<String, Value>,
    parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Scope) -> Self {
        Self {
            vars: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.vars
            .get(name)
            .or_else(|| self.parent.as_ref().and_then(|p| p.get(name)))
    }

    pub fn set(&mut self, name: String, value: Value) {
        if self.vars.contains_key(&name) {
            self.vars.insert(name, value);
        } else if let Some(ref mut parent) = self.parent {
            parent.set(name, value);
        } else {
            self.vars.insert(name, value);
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.vars.insert(name, value);
    }

    pub fn contains(&self, name: &str) -> bool {
        self.vars.contains_key(name)
            || self
                .parent
                .as_ref()
                .map(|p| p.contains(name))
                .unwrap_or(false)
    }

    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    pub fn depth(&self) -> usize {
        match &self.parent {
            Some(p) => 1 + p.depth(),
            None => 0,
        }
    }

    pub fn local_keys(&self) -> Vec<String> {
        self.vars.keys().cloned().collect()
    }

    pub fn local_count(&self) -> usize {
        self.vars.len()
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_scope() {
        let scope = Scope::new();
        assert!(scope.is_root());
        assert_eq!(scope.depth(), 0);
        assert_eq!(scope.local_count(), 0);
    }

    #[test]
    fn test_default_scope() {
        let scope = Scope::default();
        assert!(scope.is_root());
    }

    #[test]
    fn test_define_and_get() {
        let mut scope = Scope::new();
        scope.define("x".to_string(), Value::Number(42.0));
        assert_eq!(scope.get("x"), Some(&Value::Number(42.0)));
    }

    #[test]
    fn test_get_missing() {
        let scope = Scope::new();
        assert_eq!(scope.get("missing"), None);
    }

    #[test]
    fn test_with_parent() {
        let mut parent = Scope::new();
        parent.define("x".to_string(), Value::Number(1.0));

        let child = Scope::with_parent(parent);
        assert!(!child.is_root());
        assert_eq!(child.depth(), 1);
        assert_eq!(child.get("x"), Some(&Value::Number(1.0)));
    }

    #[test]
    fn test_child_shadows_parent() {
        let mut parent = Scope::new();
        parent.define("x".to_string(), Value::Number(1.0));

        let mut child = Scope::with_parent(parent);
        child.define("x".to_string(), Value::Number(99.0));
        assert_eq!(child.get("x"), Some(&Value::Number(99.0)));
    }

    #[test]
    fn test_nested_scopes() {
        let mut root = Scope::new();
        root.define("a".to_string(), Value::Number(1.0));

        let mut mid = Scope::with_parent(root);
        mid.define("b".to_string(), Value::Number(2.0));

        let leaf = Scope::with_parent(mid);
        assert_eq!(leaf.depth(), 2);
        assert_eq!(leaf.get("a"), Some(&Value::Number(1.0)));
        assert_eq!(leaf.get("b"), Some(&Value::Number(2.0)));
        assert_eq!(leaf.get("c"), None);
    }

    #[test]
    fn test_set_existing_in_parent() {
        let mut parent = Scope::new();
        parent.define("x".to_string(), Value::Number(1.0));

        let mut child = Scope::with_parent(parent);
        child.set("x".to_string(), Value::Number(99.0));
        // set should update in parent since child doesn't define "x"
        assert_eq!(child.get("x"), Some(&Value::Number(99.0)));
    }

    #[test]
    fn test_set_new_in_leaf() {
        let parent = Scope::new();
        let mut child = Scope::with_parent(parent);
        child.set("x".to_string(), Value::Number(1.0));
        // set on new key when no parent defines it should add to current scope
        assert!(child.contains("x"));
    }

    #[test]
    fn test_contains() {
        let mut parent = Scope::new();
        parent.define("x".to_string(), Value::Number(1.0));

        let child = Scope::with_parent(parent);
        assert!(child.contains("x"));
        assert!(!child.contains("y"));
    }

    #[test]
    fn test_local_keys() {
        let mut scope = Scope::new();
        scope.define("a".to_string(), Value::Null);
        scope.define("b".to_string(), Value::Null);
        let mut keys = scope.local_keys();
        keys.sort();
        assert_eq!(keys, vec!["a", "b"]);
    }

    #[test]
    fn test_local_keys_excludes_parent() {
        let mut parent = Scope::new();
        parent.define("x".to_string(), Value::Null);

        let mut child = Scope::with_parent(parent);
        child.define("y".to_string(), Value::Null);

        let child_keys = child.local_keys();
        assert_eq!(child_keys, vec!["y"]);
    }

    #[test]
    fn test_local_count() {
        let mut scope = Scope::new();
        scope.define("a".to_string(), Value::Null);
        scope.define("b".to_string(), Value::Null);
        assert_eq!(scope.local_count(), 2);
    }

    #[test]
    fn test_clone() {
        let mut scope = Scope::new();
        scope.define("x".to_string(), Value::Number(42.0));
        let cloned = scope.clone();
        assert_eq!(cloned.get("x"), Some(&Value::Number(42.0)));
    }
}
