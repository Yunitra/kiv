//! Variable storage strategies.

use inkwell::values::{BasicValueEnum, PointerValue};

/// Variable storage strategy
pub(super) enum VarStorage<'ctx> {
    /// SSA value (for Copy types: Int, Float, Bool)
    Ssa(BasicValueEnum<'ctx>),
    /// Stack allocation (for CoW types: Text)
    #[allow(dead_code)] // Reserved for future CoW type support
    Alloca(PointerValue<'ctx>),
}
