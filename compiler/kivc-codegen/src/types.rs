//! Type mapping from Kiv types to LLVM types.

use crate::error::{CodegenError, CodegenResult};
use inkwell::context::Context;
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum};
use inkwell::AddressSpace;
use kivc_typeck::Type;

/// Maps a Kiv type to an LLVM type
#[allow(dead_code)]
pub fn type_to_llvm<'ctx>(context: &'ctx Context, ty: &Type) -> CodegenResult<BasicTypeEnum<'ctx>> {
    match ty {
        Type::Int => Ok(context.i64_type().into()),
        Type::Float => Ok(context.f64_type().into()),
        Type::Bool => Ok(context.bool_type().into()),
        Type::Text => {
            // Text is represented as an opaque pointer (LLVM 18+)
            let _text_type = context.opaque_struct_type("Text");
            Ok(context.ptr_type(AddressSpace::default()).into())
        }
        Type::Unit => {
            // Unit type as i1 (bool) for compatibility - it's actually void but BasicTypeEnum can't represent void
            Ok(context.bool_type().into())
        }
        Type::Unknown => Err(CodegenError::TypeError(
            "Unknown type cannot be converted to LLVM".to_string(),
        )),
    }
}

/// Maps a Kiv type to an LLVM metadata type (for function parameters)
#[allow(dead_code)]
pub fn type_to_metadata<'ctx>(
    context: &'ctx Context,
    ty: &Type,
) -> CodegenResult<BasicMetadataTypeEnum<'ctx>> {
    type_to_llvm(context, ty).map(|t| t.into())
}

/// Returns the default LLVM value for a type (zero-initialized)
#[allow(dead_code)]
pub fn default_value<'ctx>(
    context: &'ctx Context,
    ty: &Type,
) -> CodegenResult<inkwell::values::BasicValueEnum<'ctx>> {
    match ty {
        Type::Int => Ok(context.i64_type().const_zero().into()),
        Type::Float => Ok(context.f64_type().const_zero().into()),
        Type::Bool => Ok(context.bool_type().const_zero().into()),
        Type::Text => {
            let _text_type = context.opaque_struct_type("Text");
            Ok(context
                .ptr_type(AddressSpace::default())
                .const_null()
                .into())
        }
        Type::Unit => Err(CodegenError::TypeError(
            "Unit type has no default value".to_string(),
        )),
        Type::Unknown => Err(CodegenError::TypeError(
            "Unknown type has no default value".to_string(),
        )),
    }
}
