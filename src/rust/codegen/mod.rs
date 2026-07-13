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
        for item in &program.items {
            if let Item::Fn(f, _) = item {
                self.output.push_str("fn ");
                self.output.push_str(&f.name);
                self.output.push_str("()");
                if let Some(ref _ret) = f.return_type {
                    self.output.push_str(" -> ");
                }
                if let Some(ref body) = f.body {
                    self.output.push_str(" {\n");
                    for _s in &body.stmts {
                        self.output.push_str("    ...\n");
                    }
                    self.output.push_str("}\n");
                }
            }
        }
        self.output.clone()
    }
}
