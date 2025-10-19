//! Function and parameter formatting.

use super::Formatter;
use kivc_ast::{FunDef, Param, Type, TypeKind};
use std::fmt::Write as FmtWrite;

impl Formatter {
    /// Formats a function definition
    pub(super) fn format_function(&mut self, func: &FunDef) {
        write!(self.output, "fun {}(", func.name).unwrap();

        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.format_param(param);
        }

        self.output.push(')');

        if let Some(ref ret_ty) = func.ret_ty {
            self.output.push_str(": ");
            self.format_type(ret_ty);
        }

        self.output.push_str(" {\n");

        self.indent_level += 1;
        self.format_block(&func.body);
        self.indent_level -= 1;

        self.output.push_str("}\n");
    }

    /// Formats a parameter
    fn format_param(&mut self, param: &Param) {
        write!(self.output, "{}: ", param.name).unwrap();
        self.format_type(&param.ty);
    }

    /// Formats a type
    pub(super) fn format_type(&mut self, ty: &Type) {
        let type_str = match ty.kind {
            TypeKind::Int => "Int",
            TypeKind::Float => "Float",
            TypeKind::Bool => "Bool",
            TypeKind::Text => "Text",
            TypeKind::Unit => "()",
        };
        self.output.push_str(type_str);
    }
}
