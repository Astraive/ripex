use crate::js::ast::Program;

pub trait TransformPass {
    fn name(&self) -> &'static str;
    fn run(&mut self, program: &mut Program);
}
