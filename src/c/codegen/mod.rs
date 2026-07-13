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
        for stmt in &program.decls {
            if let Stmt::Decl(f, _) = stmt {
                self.output.push_str("int ");
                self.output.push_str(&f.name);
                self.output.push_str("() {}\n");
            }
        }
        self.output.clone()
    }
}
