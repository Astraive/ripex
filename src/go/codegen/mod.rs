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
        Self {
            output: String::new(),
            indent: 0,
        }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        self.output.clear();
        self.indent = 0;
        for decl in &program.decls {
            self.emit_decl(decl);
        }
        self.output.clone()
    }

    fn push_indent(&mut self) {
        self.output.push_str(&"\t".repeat(self.indent));
    }

    fn emit_decl(&mut self, decl: &Decl) {
        self.push_indent();
        match decl {
            Decl::Package(name, _) => {
                self.output.push_str("package ");
                self.output.push_str(name);
            }
            Decl::Import(import, _) => {
                self.output.push_str("import ");
                if let Some(alias) = &import.alias {
                    self.output.push_str(alias);
                    self.output.push(' ');
                }
                self.emit_import_path(&import.path);
            }
            Decl::ImportGroup(imports, _) => {
                self.output.push_str("import (\n");
                self.indent += 1;
                for import in imports {
                    self.push_indent();
                    if let Some(alias) = &import.alias {
                        self.output.push_str(alias);
                        self.output.push(' ');
                    }
                    self.emit_import_path(&import.path);
                    self.output.push('\n');
                }
                self.indent -= 1;
                self.push_indent();
                self.output.push(')');
            }
            Decl::Var(var, _) => {
                self.emit_value_decl("var", &var.names, var.kind.as_deref(), &var.values)
            }
            Decl::Const(value, _) => {
                self.emit_value_decl("const", &value.names, value.kind.as_deref(), &value.values)
            }
            Decl::Type(value, _) => {
                self.output.push_str("type ");
                self.output.push_str(&value.name);
                self.output.push(' ');
                self.emit_expr(&value.kind);
            }
            Decl::Func(function, _) => {
                self.output.push_str("func ");
                if let Some((name, ty)) = &function.receiver {
                    self.output.push('(');
                    self.output.push_str(name);
                    self.output.push(' ');
                    self.output.push_str(ty);
                    self.output.push_str(") ");
                }
                self.output.push_str(&function.name);
                self.emit_signature(&function.params, &function.returns);
                if let Some(body) = &function.body {
                    self.output.push(' ');
                    self.emit_block(body);
                    return;
                }
            }
        }
        self.output.push('\n');
    }

    fn emit_import_path(&mut self, path: &str) {
        if path.starts_with(['"', '`']) {
            self.output.push_str(path);
        } else {
            self.output.push_str(&format!("{path:?}"));
        }
    }

    fn emit_value_decl(
        &mut self,
        keyword: &str,
        names: &[String],
        ty: Option<&Expr>,
        values: &[Expr],
    ) {
        self.output.push_str(keyword);
        self.output.push(' ');
        self.output.push_str(&names.join(", "));
        if let Some(ty) = ty {
            self.output.push(' ');
            self.emit_expr(ty);
        }
        if !values.is_empty() {
            self.output.push_str(" = ");
            self.emit_exprs(values);
        }
    }

    fn emit_signature(&mut self, params: &[(String, Box<Expr>)], returns: &[Box<Expr>]) {
        self.output.push('(');
        for (index, (name, ty)) in params.iter().enumerate() {
            if index > 0 {
                self.output.push_str(", ");
            }
            if !name.is_empty() {
                self.output.push_str(name);
                self.output.push(' ');
            }
            self.emit_expr(ty);
        }
        self.output.push(')');
        match returns {
            [] => {}
            [single] => {
                self.output.push(' ');
                self.emit_expr(single);
            }
            many => {
                self.output.push_str(" (");
                for (index, ty) in many.iter().enumerate() {
                    if index > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_expr(ty);
                }
                self.output.push(')');
            }
        }
    }

    fn emit_block(&mut self, block: &Block) {
        self.output.push_str("{\n");
        self.indent += 1;
        for stmt in &block.stmts {
            self.emit_stmt(stmt);
        }
        self.indent -= 1;
        self.push_indent();
        self.output.push_str("}\n");
    }

    fn emit_stmt(&mut self, stmt: &Stmt) {
        self.push_indent();
        match stmt {
            Stmt::Expr(expr, _) => self.emit_expr(expr),
            Stmt::Decl(decl, _) => {
                self.output
                    .truncate(self.output.trim_end_matches(['\t']).len());
                self.emit_decl(decl);
                return;
            }
            Stmt::Assign(left, right, _) => {
                self.emit_exprs(left);
                self.output.push_str(" = ");
                self.emit_exprs(right);
            }
            Stmt::Define(left, right, _) => {
                self.emit_exprs(left);
                self.output.push_str(" := ");
                self.emit_exprs(right);
            }
            Stmt::If(test, body, alternate, _) => {
                self.output.push_str("if ");
                self.emit_expr(test);
                self.output.push(' ');
                self.emit_stmt_body(body);
                if let Some(alternate) = alternate {
                    self.output.pop();
                    self.output.push_str(" else ");
                    self.emit_stmt_body(alternate);
                }
                return;
            }
            Stmt::For(init, test, update, body, _) => {
                self.output.push_str("for ");
                if init.is_some() || update.is_some() {
                    if let Some(init) = init {
                        self.emit_inline_stmt(init);
                    }
                    self.output.push_str("; ");
                    if let Some(test) = test {
                        self.emit_expr(test);
                    }
                    self.output.push_str("; ");
                    if let Some(update) = update {
                        self.emit_inline_stmt(update);
                    }
                } else if let Some(test) = test {
                    self.emit_expr(test);
                }
                self.output.push(' ');
                self.emit_stmt_body(body);
                return;
            }
            Stmt::ForRange(iter, first, second, body, _) => {
                self.output.push_str("for ");
                if first.is_empty() {
                    self.output.push_str("range ");
                } else {
                    self.output.push_str(first);
                    if let Some(second) = second {
                        self.output.push_str(", ");
                        self.output.push_str(second);
                    }
                    self.output.push_str(" := range ");
                }
                self.emit_expr(iter);
                self.output.push(' ');
                self.emit_stmt_body(body);
                return;
            }
            Stmt::Switch(value, cases, _) => {
                self.output.push_str("switch");
                if let Some(value) = value {
                    self.output.push(' ');
                    self.emit_expr(value);
                }
                self.emit_cases(cases);
                return;
            }
            Stmt::Select(cases, _) => {
                self.output.push_str("select");
                self.emit_cases(cases);
                return;
            }
            Stmt::Return(values, _) => {
                self.output.push_str("return");
                if !values.is_empty() {
                    self.output.push(' ');
                    self.emit_exprs(values);
                }
            }
            Stmt::Break(label, _) => {
                self.output.push_str("break");
                if let Some(label) = label {
                    self.output.push(' ');
                    self.output.push_str(label);
                }
            }
            Stmt::Continue(label, _) => {
                self.output.push_str("continue");
                if let Some(label) = label {
                    self.output.push(' ');
                    self.output.push_str(label);
                }
            }
            Stmt::Defer(expr, _) => {
                self.output.push_str("defer ");
                self.emit_expr(expr);
            }
            Stmt::Go(expr, _) => {
                self.output.push_str("go ");
                self.emit_expr(expr);
            }
            Stmt::Block(block, _) => {
                self.emit_block(block);
                return;
            }
            Stmt::Empty(_) => {}
            Stmt::Label(label, body, _) => {
                self.output.push_str(label);
                self.output.push_str(":\n");
                self.emit_stmt(body);
                return;
            }
            Stmt::Goto(label, _) => {
                self.output.push_str("goto ");
                self.output.push_str(label);
            }
            Stmt::Send(channel, value, _) => {
                self.emit_expr(channel);
                self.output.push_str(" <- ");
                self.emit_expr(value);
            }
            Stmt::Fallthrough(_) => self.output.push_str("fallthrough"),
        }
        self.output.push('\n');
    }

    fn emit_stmt_body(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(block, _) => self.emit_block(block),
            other => {
                self.output.push_str("{\n");
                self.indent += 1;
                self.emit_stmt(other);
                self.indent -= 1;
                self.push_indent();
                self.output.push_str("}\n");
            }
        }
    }

    fn emit_inline_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr, _) => self.emit_expr(expr),
            Stmt::Assign(left, right, _) => {
                self.emit_exprs(left);
                self.output.push_str(" = ");
                self.emit_exprs(right);
            }
            Stmt::Define(left, right, _) => {
                self.emit_exprs(left);
                self.output.push_str(" := ");
                self.emit_exprs(right);
            }
            Stmt::Decl(Decl::Var(var, _), _) => {
                self.emit_value_decl("var", &var.names, var.kind.as_deref(), &var.values)
            }
            _ => {}
        }
    }

    fn emit_cases(&mut self, cases: &[CaseClause]) {
        self.output.push_str(" {\n");
        self.indent += 1;
        for case in cases {
            self.push_indent();
            if let Some(expr) = &case.expr {
                self.output.push_str("case ");
                self.emit_expr(expr);
            } else {
                self.output.push_str("default");
            }
            self.output.push_str(":\n");
            self.indent += 1;
            for stmt in &case.body {
                self.emit_stmt(stmt);
            }
            self.indent -= 1;
        }
        self.indent -= 1;
        self.push_indent();
        self.output.push_str("}\n");
    }

    fn emit_exprs(&mut self, exprs: &[Expr]) {
        for (index, expr) in exprs.iter().enumerate() {
            if index > 0 {
                self.output.push_str(", ");
            }
            self.emit_expr(expr);
        }
    }

    fn emit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Bool(value, _) => self.output.push_str(if *value { "true" } else { "false" }),
            Expr::Int(value, _) => self.output.push_str(&value.to_string()),
            Expr::Float(value, _) => self.output.push_str(&value.to_string()),
            Expr::String(value, _) => {
                if value.starts_with(['"', '`']) {
                    self.output.push_str(value);
                } else {
                    self.output.push_str(&format!("{value:?}"));
                }
            }
            Expr::Nil(_) => self.output.push_str("nil"),
            Expr::Ident(name, _) => self.output.push_str(name),
            Expr::Binary(left, op, right, _) => {
                self.output.push('(');
                self.emit_expr(left);
                self.output.push(' ');
                self.output.push_str(binary_op(*op));
                self.output.push(' ');
                self.emit_expr(right);
                self.output.push(')');
            }
            Expr::Unary(op, value, _) => {
                self.output.push_str(unary_op(*op));
                self.emit_expr(value);
            }
            Expr::Call(function, args, _) => {
                self.emit_expr(function);
                self.output.push('(');
                self.emit_exprs(args);
                self.output.push(')');
            }
            Expr::Index(object, index, _) => {
                self.emit_expr(object);
                self.output.push('[');
                self.emit_expr(index);
                self.output.push(']');
            }
            Expr::Selector(object, property, _) => {
                self.emit_expr(object);
                self.output.push('.');
                self.output.push_str(property);
            }
            Expr::Slice(object, low, high, _) => {
                self.emit_expr(object);
                self.output.push('[');
                if let Some(low) = low {
                    self.emit_expr(low);
                }
                self.output.push(':');
                if let Some(high) = high {
                    self.emit_expr(high);
                }
                self.output.push(']');
            }
            Expr::Array(items, _) => {
                self.output.push_str("[]any{");
                self.emit_exprs(items);
                self.output.push('}');
            }
            Expr::StructLit(name, fields, _) => {
                self.output.push_str(name);
                self.output.push('{');
                for (index, field) in fields.iter().enumerate() {
                    if index > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(&field.name);
                    if let Some(value) = &field.value {
                        self.output.push_str(": ");
                        self.emit_expr(value);
                    }
                }
                self.output.push('}');
            }
            Expr::MapLit(items, _) => {
                self.output.push_str("map[any]any{");
                for (index, (key, value)) in items.iter().enumerate() {
                    if index > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_expr(key);
                    self.output.push_str(": ");
                    self.emit_expr(value);
                }
                self.output.push('}');
            }
            Expr::FuncLit(signature, body, _) => {
                self.output.push_str("func");
                self.emit_signature(&signature.params, &signature.returns);
                self.output.push(' ');
                self.emit_block(body);
                self.output.pop();
            }
            Expr::Paren(value, _) => {
                self.output.push('(');
                self.emit_expr(value);
                self.output.push(')');
            }
            Expr::TypeAssert(value, ty, _) => {
                self.emit_expr(value);
                self.output.push_str(".(");
                self.emit_expr(ty);
                self.output.push(')');
            }
            Expr::CompositeLit(ty, items, _) => {
                self.emit_expr(ty);
                self.output.push('{');
                self.emit_exprs(items);
                self.output.push('}');
            }
        }
    }
}

fn binary_op(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Mod => "%",
        BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Gt => ">",
        BinaryOp::Le => "<=",
        BinaryOp::Ge => ">=",
        BinaryOp::And => "&&",
        BinaryOp::Or => "||",
        BinaryOp::BitAnd => "&",
        BinaryOp::BitOr => "|",
        BinaryOp::BitXor => "^",
        BinaryOp::BitClear => "&^",
        BinaryOp::Shl => "<<",
        BinaryOp::Shr => ">>",
        BinaryOp::Assign => "=",
        BinaryOp::AddAssign => "+=",
        BinaryOp::SubAssign => "-=",
        BinaryOp::MulAssign => "*=",
        BinaryOp::DivAssign => "/=",
        BinaryOp::ModAssign => "%=",
        BinaryOp::AndAssign => "&=",
        BinaryOp::OrAssign => "|=",
        BinaryOp::XorAssign => "^=",
        BinaryOp::ShlAssign => "<<=",
        BinaryOp::ShrAssign => ">>=",
        BinaryOp::BitClearAssign => "&^=",
    }
}

fn unary_op(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Neg => "-",
        UnaryOp::Not => "!",
        UnaryOp::Deref => "*",
        UnaryOp::Ref => "&",
        UnaryOp::Receive => "<-",
        UnaryOp::Plus => "+",
    }
}
