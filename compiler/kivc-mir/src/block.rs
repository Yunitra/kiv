//! Basic block management for MIR.

use crate::mir::{MirInstr, MirTerminator};

/// A unique identifier for a basic block
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

impl BlockId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

/// A basic block: a sequence of instructions with a single entry and exit
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Unique identifier for this block
    pub id: BlockId,

    /// Instructions in this block (no control flow)
    pub instructions: Vec<MirInstr>,

    /// Terminator (control flow at the end of the block)
    pub terminator: MirTerminator,
}

impl BasicBlock {
    /// Creates a new basic block
    pub fn new(id: BlockId, terminator: MirTerminator) -> Self {
        Self {
            id,
            instructions: Vec::new(),
            terminator,
        }
    }

    /// Adds an instruction to this block
    pub fn push_instr(&mut self, instr: MirInstr) {
        self.instructions.push(instr);
    }
}

/// A builder for constructing basic blocks
#[derive(Debug)]
pub struct BlockBuilder {
    next_id: usize,
}

impl BlockBuilder {
    /// Creates a new block builder
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    /// Allocates a new block ID
    pub fn alloc_block_id(&mut self) -> BlockId {
        let id = BlockId::new(self.next_id);
        self.next_id += 1;
        id
    }
}

impl Default for BlockBuilder {
    fn default() -> Self {
        Self::new()
    }
}
