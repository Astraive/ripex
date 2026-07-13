use crate::js::ast::Program;

use crate::js::transform::pass::TransformPass;

pub struct TypeScriptTransform;

impl Default for TypeScriptTransform {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeScriptTransform {
    pub fn new() -> Self {
        TypeScriptTransform
    }
}

impl TransformPass for TypeScriptTransform {
    fn name(&self) -> &'static str {
        "typescript-transform"
    }

    fn run(&mut self, _program: &mut Program) {
        // Strip TypeScript type annotations
    }
}
