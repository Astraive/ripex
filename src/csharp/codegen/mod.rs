use super::ast::*;

pub struct Codegen {
    output: String,
}

impl Default for Codegen {
    fn default() -> Self {
        Self::new()
    }
}

impl Codegen {
    pub fn new() -> Self {
        Codegen {
            output: String::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        self.output.clear();
        for decl in &program.decls {
            if let Decl::Class(c, _) = decl {
                self.output.push_str("class ");
                self.output.push_str(&c.name);
                self.output.push_str(" { }");
            }
        }
        self.output.clone()
    }
}
