//! Mid-level Intermediate Representation (MIR) for the Kiv compiler.
//!
//! MIR is a simplified, linearized representation of the HIR that is closer
//! to machine code. It uses basic blocks with explicit control flow.

mod block;
mod lower;
mod mir;

pub use block::{BasicBlock, BlockId};
pub use lower::lower_to_mir;
pub use mir::{MirFunction, MirInstr, MirOperand, MirProgram, MirTerminator};

// Re-export commonly used types from dependencies
pub use kivc_ast::{BinOp, Literal};
pub use kivc_hir::VarId;
