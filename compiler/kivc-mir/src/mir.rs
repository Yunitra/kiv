//! MIR node definitions.

use crate::block::{BasicBlock, BlockId};
use kivc_ast::{BinOp, Literal};
use kivc_hir::{FunId, TypeId, VarId};
use kivc_typeck::Type;
use std::collections::HashMap;

/// A complete MIR program
#[derive(Debug, Clone)]
pub struct MirProgram {
    pub functions: Vec<MirFunction>,
    pub type_table: HashMap<TypeId, Type>,
}

/// A function in MIR
#[derive(Debug, Clone)]
pub struct MirFunction {
    pub id: FunId,
    pub name: String,
    pub params: Vec<VarId>,
    pub param_types: Vec<TypeId>, // NEW: parameter types
    pub return_type: Option<TypeId>,
    pub blocks: Vec<BasicBlock>,
    pub entry_block: BlockId,
}

/// An instruction in MIR (no control flow)
#[derive(Debug, Clone)]
pub enum MirInstr {
    /// Assign a value to a variable
    Assign { dest: VarId, source: MirOperand },

    /// Binary operation
    BinOp {
        dest: VarId,
        op: BinOp,
        left: MirOperand,
        right: MirOperand,
    },

    /// Function call
    Call {
        dest: Option<VarId>,
        func: String,
        args: Vec<MirOperand>,
    },

    /// No-op (for debugging or placeholder)
    Nop,
}

/// An operand in MIR (a value)
#[derive(Debug, Clone)]
pub enum MirOperand {
    /// A literal value
    Literal(Literal),

    /// A variable reference
    Var(VarId),

    /// Unit value
    Unit,
}

/// A terminator (control flow at the end of a basic block)
#[derive(Debug, Clone)]
pub enum MirTerminator {
    /// Unconditional jump to a block
    Jump { target: BlockId },

    /// Conditional branch
    Branch {
        condition: MirOperand,
        true_block: BlockId,
        false_block: BlockId,
    },

    /// Return from function
    Return { value: Option<MirOperand> },

    /// Unreachable (for error paths)
    Unreachable,
}

impl MirProgram {
    /// Creates a new MIR program
    pub fn new(functions: Vec<MirFunction>, type_table: HashMap<TypeId, Type>) -> Self {
        Self {
            functions,
            type_table,
        }
    }
}

impl MirFunction {
    /// Creates a new MIR function
    pub fn new(
        id: FunId,
        name: String,
        params: Vec<VarId>,
        param_types: Vec<TypeId>,
        return_type: Option<TypeId>,
        entry_block: BlockId,
    ) -> Self {
        Self {
            id,
            name,
            params,
            param_types,
            return_type,
            blocks: Vec::new(),
            entry_block,
        }
    }

    /// Adds a basic block to this function
    pub fn add_block(&mut self, block: BasicBlock) {
        self.blocks.push(block);
    }

    /// Gets a block by ID
    pub fn get_block(&self, id: BlockId) -> Option<&BasicBlock> {
        self.blocks.iter().find(|b| b.id == id)
    }

    /// Gets a mutable block by ID
    pub fn get_block_mut(&mut self, id: BlockId) -> Option<&mut BasicBlock> {
        self.blocks.iter_mut().find(|b| b.id == id)
    }
}
