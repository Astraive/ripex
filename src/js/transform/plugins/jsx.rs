use crate::js::ast::Program;

use crate::js::transform::pass::TransformPass;

pub struct JsxTransform;

impl Default for JsxTransform {
    fn default() -> Self {
        Self::new()
    }
}

impl JsxTransform {
    pub fn new() -> Self {
        JsxTransform
    }
}

impl TransformPass for JsxTransform {
    fn name(&self) -> &'static str {
        "jsx-transform"
    }

    fn run(&mut self, _program: &mut Program) {
        // Transform JSX to React.createElement calls
    }
}
