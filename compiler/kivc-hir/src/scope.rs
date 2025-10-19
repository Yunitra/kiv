//! Scope management for name resolution.

use crate::id::VarId;
use std::collections::HashMap;

/// A stack of scopes for variable name resolution
#[derive(Debug, Clone)]
pub struct ScopeStack {
    scopes: Vec<Scope>,
    next_var_id: usize,
}

/// A single scope containing variable bindings
#[derive(Debug, Clone)]
struct Scope {
    bindings: HashMap<String, VarId>,
}

impl ScopeStack {
    /// Creates a new empty scope stack
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
            next_var_id: 0,
        }
    }

    /// Pushes a new scope onto the stack
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    /// Pops the current scope from the stack
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Declares a new variable in the current scope
    pub fn declare(&mut self, name: String) -> VarId {
        let var_id = VarId::new(self.next_var_id);
        self.next_var_id += 1;

        if let Some(scope) = self.scopes.last_mut() {
            scope.bindings.insert(name, var_id);
        }

        var_id
    }

    /// Looks up a variable by name, searching from innermost to outermost scope
    pub fn lookup(&self, name: &str) -> Option<VarId> {
        for scope in self.scopes.iter().rev() {
            if let Some(&var_id) = scope.bindings.get(name) {
                return Some(var_id);
            }
        }
        None
    }

    /// Checks if a variable is declared in the current scope (not parent scopes)
    pub fn is_declared_in_current(&self, name: &str) -> bool {
        if let Some(scope) = self.scopes.last() {
            scope.bindings.contains_key(name)
        } else {
            false
        }
    }
}

impl Scope {
    fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }
}

impl Default for ScopeStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_basic() {
        let mut scopes = ScopeStack::new();

        // Declare variable in outer scope
        let x_id = scopes.declare("x".to_string());
        assert_eq!(scopes.lookup("x"), Some(x_id));
    }

    #[test]
    fn test_scope_shadowing() {
        let mut scopes = ScopeStack::new();

        // Declare x in outer scope
        let x_outer = scopes.declare("x".to_string());

        // Push new scope and declare x again
        scopes.push_scope();
        let x_inner = scopes.declare("x".to_string());

        // Inner x shadows outer x
        assert_eq!(scopes.lookup("x"), Some(x_inner));
        assert_ne!(x_inner, x_outer);

        // Pop scope, outer x is visible again
        scopes.pop_scope();
        assert_eq!(scopes.lookup("x"), Some(x_outer));
    }

    #[test]
    fn test_scope_not_found() {
        let scopes = ScopeStack::new();
        assert_eq!(scopes.lookup("nonexistent"), None);
    }

    #[test]
    fn test_scope_nested() {
        let mut scopes = ScopeStack::new();

        let a = scopes.declare("a".to_string());

        scopes.push_scope();
        let b = scopes.declare("b".to_string());

        scopes.push_scope();
        let c = scopes.declare("c".to_string());

        // All variables visible
        assert_eq!(scopes.lookup("a"), Some(a));
        assert_eq!(scopes.lookup("b"), Some(b));
        assert_eq!(scopes.lookup("c"), Some(c));

        scopes.pop_scope();
        // c no longer visible
        assert_eq!(scopes.lookup("c"), None);
        assert_eq!(scopes.lookup("a"), Some(a));
        assert_eq!(scopes.lookup("b"), Some(b));
    }
}
