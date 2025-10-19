//! Unique identifiers for variables, functions, and types.

/// A unique identifier for a variable
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VarId(pub usize);

/// A unique identifier for a function
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunId(pub usize);

/// A unique identifier for a type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub usize);

impl VarId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl FunId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl TypeId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}
