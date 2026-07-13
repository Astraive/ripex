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
            match decl {
                Decl::Package(name, _) => {
                    self.output.push_str("package ");
                    self.output.push_str(name);
                    self.output.push('\n');
                }
                Decl::Import(_, _) => {}
                Decl::Func(f, _) => {
                    self.output.push_str("func ");
                    self.output.push_str(&f.name);
                    self.output.push('(');
                    for (i, (n, _)) in f.params.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.output.push_str(n);
                    }
                    self.output.push(')');
                    self.output.push_str(" {");
                    if let Some(ref body) = f.body {
                        for _s in &body.stmts {
                            self.output.push_str(" ... ");
                        }
                    }
                    self.output.push_str("}\n");
                }
                _ => {}
            }
        }
        self.output.clone()
    }
}
