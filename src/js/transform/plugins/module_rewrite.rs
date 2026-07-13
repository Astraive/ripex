use crate::js::ast::Program;

use crate::js::transform::pass::TransformPass;

pub struct ModuleRewriteTransform;

impl Default for ModuleRewriteTransform {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleRewriteTransform {
    pub fn new() -> Self {
        ModuleRewriteTransform
    }
}

impl TransformPass for ModuleRewriteTransform {
    fn name(&self) -> &'static str {
        "module-rewrite"
    }

    fn run(&mut self, _program: &mut Program) {
        // Rewrite ES modules to CommonJS
    }
}
