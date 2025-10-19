//! Type checker core.

use crate::types::{Type, TypeContext};
use kivc_diagnostics::DiagnosticsCollector;
use kivc_hir::{HirFunDef, HirProgram, TypeId, VarId};
use std::collections::HashMap;

/// The type checking context
pub struct TypeChecker {
    pub(super) ctx: TypeContext,
    pub(super) var_types: HashMap<VarId, Type>,
    pub(super) var_mutability: HashMap<VarId, bool>,
    pub(super) current_return_type: Option<TypeId>,
    pub(super) in_loop_depth: usize,
    pub(super) diagnostics: DiagnosticsCollector,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            ctx: TypeContext::new(),
            var_types: HashMap::new(),
            var_mutability: HashMap::new(),
            current_return_type: None,
            in_loop_depth: 0,
            diagnostics: DiagnosticsCollector::new(),
        }
    }

    pub fn check_program(&mut self, program: HirProgram) -> HirProgram {
        let functions = program
            .functions
            .into_iter()
            .map(|f| self.check_function(f))
            .collect();

        HirProgram { functions }
    }

    fn check_function(&mut self, mut fun: HirFunDef) -> HirFunDef {
        // Store parameter types
        for param in &fun.params {
            if let Some(ty) = self.ctx.get(param.ty) {
                self.var_types.insert(param.var_id, ty.clone());
                self.var_mutability.insert(param.var_id, false);
            }
        }

        // Store return type for validation
        self.current_return_type = fun.return_type;

        // Check function body
        fun.body = self.check_block(fun.body);

        // Clear function-specific state
        self.var_types.clear();
        self.var_mutability.clear();
        self.current_return_type = None;

        fun
    }
}
