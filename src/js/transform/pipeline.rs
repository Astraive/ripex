use crate::js::ast::Program;

use super::pass::TransformPass;

pub struct TransformPipeline {
    passes: Vec<Box<dyn TransformPass>>,
}

impl TransformPipeline {
    pub fn new() -> Self {
        TransformPipeline { passes: Vec::new() }
    }

    pub fn add(&mut self, pass: Box<dyn TransformPass>) {
        self.passes.push(pass);
    }

    pub fn run(&mut self, program: &mut Program) {
        for pass in &mut self.passes {
            pass.run(program);
        }
    }

    pub fn passes(&self) -> &[Box<dyn TransformPass>] {
        &self.passes
    }
}

impl Default for TransformPipeline {
    fn default() -> Self {
        TransformPipeline::new()
    }
}
