//! Runtime function declarations for LLVM.

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::AddressSpace;

/// Declares all runtime functions in the LLVM module
pub fn declare_runtime_functions<'ctx>(context: &'ctx Context, module: &Module<'ctx>) {
    let ptr_type = context.ptr_type(AddressSpace::default());
    let i64_type = context.i64_type();
    let f64_type = context.f64_type();
    let bool_type = context.bool_type();
    let void_type = context.void_type();

    // Declare all runtime functions in one block to avoid extra newlines in output
    let functions = [
        (
            "kiv_text_from_cstr",
            ptr_type.fn_type(&[ptr_type.into()], false),
        ),
        (
            "kiv_text_clone",
            ptr_type.fn_type(&[ptr_type.into()], false),
        ),
        (
            "kiv_text_drop",
            void_type.fn_type(&[ptr_type.into()], false),
        ),
        (
            "kiv_text_concat",
            ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
        ),
        (
            "kiv_text_equals",
            bool_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
        ),
        (
            "kiv_print_text",
            void_type.fn_type(&[ptr_type.into()], false),
        ),
        (
            "kiv_print_int",
            void_type.fn_type(&[i64_type.into()], false),
        ),
        (
            "kiv_print_float",
            void_type.fn_type(&[f64_type.into()], false),
        ),
        (
            "kiv_print_bool",
            void_type.fn_type(&[bool_type.into()], false),
        ),
        ("kiv_print_newline", void_type.fn_type(&[], false)),
        (
            "kiv_int_to_text",
            ptr_type.fn_type(&[i64_type.into()], false),
        ),
        (
            "kiv_float_to_text",
            ptr_type.fn_type(&[f64_type.into()], false),
        ),
        (
            "kiv_bool_to_text",
            ptr_type.fn_type(&[bool_type.into()], false),
        ),
        ("kiv_panic", void_type.fn_type(&[ptr_type.into()], false)),
    ];

    for (name, fn_type) in functions {
        module.add_function(name, fn_type, None);
    }
}
