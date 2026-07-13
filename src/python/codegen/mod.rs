// Basic Python code generator (printer)
use super::ast::*;

pub struct Codegen {
    output: String,
    indent: usize,
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
            indent: 0,
        }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        self.output.clear();
        self.indent = 0;
        for stmt in &program.stmts {
            self.emit_stmt(stmt);
        }
        self.output.clone()
    }

    fn emit_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
    }

    fn emit_stmt(&mut self, stmt: &Stmt) {
        self.emit_indent();
        match stmt {
            Stmt::Expr(e, _) => self.emit_expr(e),
            Stmt::Pass(_) => self.output.push_str("pass"),
            Stmt::Return(Some(e), _) => {
                self.output.push_str("return ");
                self.emit_expr(e);
            }
            Stmt::Return(None, _) => self.output.push_str("return"),
            Stmt::Break(_) => self.output.push_str("break"),
            Stmt::Continue(_) => self.output.push_str("continue"),
            _ => self.output.push_str("..."),
        }
        self.output.push('\n');
    }

    fn emit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(name, _) => self.output.push_str(name),
            Expr::Literal(lit, _) => match lit {
                Literal::Int(val, _, _) => self.output.push_str(&val.to_string()),
                Literal::Float(val, _, _) => self.output.push_str(&val.to_string()),
                Literal::String(text, _, _) => {
                    self.output.push('"');
                    self.output.push_str(text);
                    self.output.push('"');
                }
                Literal::Boolean(true, _) => self.output.push_str("True"),
                Literal::Boolean(false, _) => self.output.push_str("False"),
                Literal::None_(_) => self.output.push_str("None"),
                _ => self.output.push_str("..."),
            },
            Expr::Binary(left, op, right, _) => {
                self.emit_expr(left);
                let op_str = match op {
                    BinaryOp::Add => " + ",
                    BinaryOp::Sub => " - ",
                    BinaryOp::Mul => " * ",
                    BinaryOp::Div => " / ",
                    _ => " ? ",
                };
                self.output.push_str(op_str);
                self.emit_expr(right);
            }
            Expr::Call(func, args, _, _) => {
                self.emit_expr(func);
                self.output.push('(');
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_expr(a);
                }
                self.output.push(')');
            }
            _ => self.output.push_str("..."),
        }
    }
}
